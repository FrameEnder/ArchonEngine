use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use axum::Router;
use axum::routing::get;
use kawari::common::{
    BasicServiceAccountData, ClientLanguage, ContainerType, DEBUG_COMMAND_TRIGGER, FestivalId,
    GimmickAccessorType, HandlerId, HandlerType, InstanceContentType, ItemOperationKind,
    LogMessageType, ObjectId, ObjectTypeId, ObjectTypeKind, PlayerStateFlags1, PlayerStateFlags2,
    PlayerStateFlags3, Position, QuestSpecialFlags, TAB_SHARED_FATE_COUNT, TerritoryIntendedUse,
    WarpType, calculate_max_level, in_housing_area,
};
use kawari::config::get_config;
use kawari_world::inventory::{Item, MAX_LARGE_STORAGE};
use physis::equipment::EquipSlot;

use kawari::ipc::chat::ClientChatIpcData;

use kawari::ipc::zone::{
    ActorControlCategory, CWLSLeaveReason, ClientTriggerTarget, Conditions,
    ContentFinderUserAction, CrossRealmListing, CrossRealmListings, DutyFinderSetting,
    DutySupportInformation, EventLogMessage, EventType, FurnitureTranslatedForObserver, ItemInfo,
    LinkshellInviteResponse, OnlineStatus, OnlineStatusMask, PlayerSetup, SceneFlags, SearchInfo,
    SocialListRequestType, TrustContent, TrustInformation,
};

use kawari::ipc::zone::{
    Blacklist, BlacklistedCharacter, ClientTriggerCommand, ClientZoneIpcData,
    FellowshipActivityTag, FellowshipSearchInfo, ReadyCheckReply, ServerZoneIpcData,
    ServerZoneIpcSegment, SocialListUILanguages,
};

use kawari::common::{CharacterMode, NETWORK_TIMEOUT, RECEIVE_BUFFER_SIZE};

/// A quest battle's opening scene flags unless its script sets `OPENING_SCENE_FLAGS`: the most common on retail
/// (quest battles 11, 12, 15, 35 and 36 in Bob's Chronofoil captures; 14, 24 and 29 have their own.)
const QUEST_BATTLE_OPENING_FLAGS: u32 = 0xF841AEFB;

/// How often a player's progress is saved while they play, besides when they log out.
/// TeleportStart's value for a teleport the character can't afford (Sapphire's; retail sends 0 when they can.)
const TELEPORT_INSUFFICIENT_GIL: u32 = 2;

const AUTOSAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

/// How long a zone connection can go without hearing from the client before it's treated as dropped (and the player
/// is logged out and saved.)
const DEAD_CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
use kawari::constants::{
    AETHER_CURRENT_COMP_FLG_SET_BITMASK_SIZE, CLASSJOB_ARRAY_SIZE,
    DISCOVERY_MAPS_WITH_UP_TO_16_REGIONS_BITMASK_SIZE,
    DISCOVERY_MAPS_WITH_UP_TO_32_REGIONS_BITMASK_SIZE,
};
use kawari::packet::oodle::OodleNetwork;
use kawari::packet::{ConnectionState, ConnectionType, SegmentData, parse_packet_header};
use kawari_world::lua::{KawariLua, KawariLuaState, LuaPlayer};
use kawari_world::retainer;
use kawari_world::{
    BotPartyAction, ChatConnectionChannels, ChatPlayerData, ClientHandle, ClientId, FromServer,
    MessageInfo, PlayerData, ServerHandle, ToServer, WorldDatabase, server_main_loop,
};
use kawari_world::{
    CLIENT_QUEUE_SIZE, ChatConnection, CustomIpcConnection, Event, EventHandler, GameData,
    ObsfucationData, QUEST_BATTLE_SCENE_COMPLETE, QUEST_BATTLE_SCENE_FINISH,
    QUEST_BATTLE_SCENE_FINISH_FLAGS, RIDE_SHOOTING_OPENING_FLAGS, RIDE_SHOOTING_OPENING_VALUE,
    Roulette, TeleportReason, ZoneConnection,
};

use mlua::Function;
use parking_lot::Mutex;
use tokio::io::AsyncReadExt;
use tokio::join;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Receiver, UnboundedReceiver, UnboundedSender, channel, unbounded_channel};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use kawari::common::INVENTORY_ACTION_ACK_GENERAL;

fn spawn_main_loop(
    game_data: Arc<Mutex<GameData>>,
    database: Arc<Mutex<WorldDatabase>>,
) -> (ServerHandle, JoinHandle<()>) {
    let (send, recv) = channel(64);

    let handle = ServerHandle {
        chan: send,
        next_id: Default::default(),
    };

    let join = tokio::spawn(async move {
        let game_data_new;
        {
            // We let it clone our GameData so it doesn't take 2x the time to load.
            let game_data_mutex = game_data.lock();
            game_data_new = game_data_mutex.clone();
        }
        let parties;
        let linkshells;
        {
            let mut database = database.lock();
            parties = database.get_parties();
            linkshells = database.find_all_linkshells();
        }
        let res = server_main_loop(game_data_new, parties, linkshells, recv).await;
        match res {
            Ok(()) => {}
            Err(err) => {
                tracing::error!("{}", err);
            }
        }
    });

    (handle, join)
}

/// A task that will process the first few packets from each connection.
fn spawn_initial_setup(
    id: ClientId,
    socket: TcpStream,
    lua: Arc<Mutex<KawariLua>>,
    database: Arc<Mutex<WorldDatabase>>,
    game_data: Arc<Mutex<GameData>>,
    handle: ServerHandle,
) {
    let _kill = tokio::spawn(initial_setup(id, socket, lua, database, game_data, handle));
}

/// The initial setup loop, which figures out what the remote connection wants and branches off to provide a chat connection, zone connection, or custom IPC connection.
async fn initial_setup(
    id: ClientId,
    mut socket: TcpStream,
    lua: Arc<Mutex<KawariLua>>,
    database: Arc<Mutex<WorldDatabase>>,
    game_data: Arc<Mutex<GameData>>,
    handle: ServerHandle,
) {
    let mut buf = vec![0; RECEIVE_BUFFER_SIZE];

    match socket.read(&mut buf).await {
        Ok(n) => {
            let header = parse_packet_header(&buf[..n]);
            if header.connection_type == ConnectionType::KawariIpc {
                let mut connection = CustomIpcConnection {
                    socket,
                    state: ConnectionState::None,
                    database: database.clone(),
                    game_data: game_data.clone(),
                    id,
                    handle,
                };
                // Handle the first batch of segments before handing off control to the loop proper.
                let segments = connection.parse_packet(&buf[..n]);
                for segment in segments {
                    match &segment.data {
                        SegmentData::KawariIpc(data) => connection.handle_custom_ipc(data).await,
                        _ => panic!(
                            "initial_setup: The KawariIpc connection type only supports KawariIpc segments! Was a mistake made somewhere? Received: {segment:#?}"
                        ),
                    }
                }

                tracing::info!(
                    "Dropping connection from {id:?} because all custom IPC segments were sent"
                )
            } else if header.connection_type == ConnectionType::Zone {
                let state = ConnectionState::Zone {
                    clientbound_oodle: OodleNetwork::new(),
                    serverbound_oodle: OodleNetwork::new(),
                    scrambler_keys: None,
                };
                let mut connection = ZoneConnection {
                    config: get_config().world,
                    socket,
                    state,
                    player_data: PlayerData::default(),
                    id,
                    handle: handle.clone(),
                    database: database.clone(),
                    lua: lua.clone(),
                    game_data: game_data.clone(),
                    last_keep_alive: Instant::now(),
                    gracefully_logged_out: false,
                    obsfucation_data: ObsfucationData::default(),
                    queued_content: None,
                    duty_support: None,
                    duty_settings: None,
                    entered_with: None,
                    queued_roulette: None,
                    entered_roulette: None,
                    bot_party_levels: Vec::new(),
                    minimum_item_level: None,
                    echo_multiplier: 1.0,
                    conditions: Conditions::default(),
                    queued_tasks: Vec::new(),
                    old_zone_id: 0,
                    old_position: Position::default(),
                    old_rotation: 0.0,
                    teleport_reason: TeleportReason::NotSpecified,
                    active_minion: 0,
                    party_id: 0,
                    rejoining_party: false,
                    login_time: None,
                    playtime_at_login: 0,
                    glamour_information: None,
                    event_handler_id: None,
                    recipe: None,
                    craft: None,
                    quick_synthesis: None,
                    materialize: None,
                    gathering: Default::default(),
                    shown_items: Default::default(),
                    inventory_shown: false,
                    is_party_leader: false,
                    synced_level: None,
                    synced_item_level: None,
                    synced_fate: None,
                    search_results: Vec::new(),
                    search_index: 0,
                    friend_results: Vec::new(),
                    friend_index: 0,
                    cwls_results: Vec::new(),
                    cwls_index: 0,
                    mail_results: Vec::new(),
                    mail_index: 0,
                    spawned_in: false,
                    offered_teleport: None,
                    is_trading: false,
                    dyeing_information: None,
                    marketboard_request_item_id: 0,
                    hide_spectator_ui: false,
                    initial_login: true,
                    fate_motivation_npcs: HashMap::new(),
                    content_handler_id: None,
                    event_action_id: 0,
                    nested_event: None,
                    quest_battle_stay: false,
                    quest_battle_finish_scene: None,
                    quest_battle_finish_flags: QUEST_BATTLE_SCENE_FINISH_FLAGS,
                    can_share_teleport: false,
                    exp_chain: (0, None),
                    called_retainer: None,
                    pending_retainer_appearance: None,
                    content_finder_condition_id: 0,
                    duty_cleared: None,
                    pending_event_text: None,
                    scenario_guide_sent: None,
                    gold_saucer_game: None,
                    gate_joined: None,
                    gate_checked: kawari::common::timestamp_secs() as u64,
                    ride_shooting_scored: false,
                    capture: kawari_world::capture::PacketCapture::new(&get_config().world, id),
                };

                // Handle setup before passing off control to the zone connection.
                let segments = connection.parse_packet(&buf[..n]);
                let mut zone_ready = false;

                for segment in segments {
                    match &segment.data {
                        SegmentData::Setup { actor_id } => {
                            // for some reason they send a string representation
                            let Ok(actor_id) = actor_id.parse::<u32>() else {
                                tracing::error!(
                                    "ZoneConnection: Client sent us an invalid actor id string in Setup! Closing connection!"
                                );
                                break;
                            };

                            // initialize player data if it doesn't exist
                            if !connection.player_data.character.actor_id.is_valid() {
                                let player_data;
                                {
                                    let mut game_data = connection.game_data.lock();
                                    let mut database = connection.database.lock();
                                    player_data = database
                                        .find_player_data(ObjectId(actor_id), &mut game_data);
                                }
                                connection.player_data = player_data;
                            }

                            // collect actor data
                            connection.initialize().await;
                            zone_ready = true;
                        }
                        _ => panic!(
                            "initial_setup: The zone connection type must start with a Setup segment! What happened? Received: {segment:#?}"
                        ),
                    }
                }

                if zone_ready {
                    spawn_client(connection);
                }
            } else if header.connection_type == ConnectionType::Chat {
                let state = ConnectionState::Zone {
                    clientbound_oodle: OodleNetwork::new(),
                    serverbound_oodle: OodleNetwork::new(),
                    scrambler_keys: None,
                };

                let mut connection = ChatConnection {
                    socket,
                    id,
                    state,
                    database: database.clone(),
                    player_data: ChatPlayerData::default(),
                    config: get_config().world,
                    last_keep_alive: Instant::now(),
                    handle,
                    chatchannels: ChatConnectionChannels::default(),
                };

                // Handle setup before passing off control to the chat connection.
                let segments = connection.parse_packet(&buf[..n]);
                let mut chat_ready = false;

                for segment in segments {
                    match &segment.data {
                        SegmentData::Setup { actor_id } => {
                            // for some reason they send a string representation
                            let Ok(actor_id) = actor_id.parse::<u32>() else {
                                tracing::error!(
                                    "ChatConnection: Client sent us an invalid actor id string in Setup! Closing connection!"
                                );
                                break;
                            };

                            if !connection.player_data.actor_id.is_valid() {
                                connection.player_data.actor_id = ObjectId(actor_id);

                                {
                                    let mut game_data = game_data.lock();
                                    let mut database = connection.database.lock();
                                    let player_data = database.find_player_data(
                                        connection.player_data.actor_id,
                                        &mut game_data,
                                    );

                                    connection.player_data.account_id =
                                        player_data.character.service_account_id as u64;
                                    connection.player_data.content_id =
                                        player_data.character.content_id as u64;
                                    connection.player_data.name =
                                        player_data.character.name.clone();
                                }

                                connection.initialize().await;
                                chat_ready = true;
                            }
                        }
                        _ => panic!(
                            "initial_setup: The chat connection type must start with a Setup segment! What happened? Received: {segment:#?}"
                        ),
                    }
                }
                if chat_ready {
                    spawn_chat_connection(connection);
                }
            } else {
                tracing::error!("Connection type is None! How did this happen?");
            }
        }
        Err(err) => {
            tracing::error!("Error while setting up connection: {err:?}");
        }
    }
}

// TODO: Is there a sensible way we can reuse the other ClientData type so we don't need 2?
struct ClientChatData {
    /// Socket for data recieved from the global server
    recv: Receiver<FromServer>,
    connection: ChatConnection,
}

/// Spawn a new chat connection for an incoming client.
fn spawn_chat_connection(connection: ChatConnection) {
    let (send, recv) = channel(CLIENT_QUEUE_SIZE);

    let id = &connection.id.clone();
    let actor_id = &connection.player_data.actor_id.clone();

    let data = ClientChatData { recv, connection };

    // Spawn a new client task
    let (my_send, my_recv) = oneshot::channel();
    let _kill = tokio::spawn(start_chat_connection(my_recv, data));

    // Send client information to said task
    let handle = ClientHandle {
        id: *id,
        channel: send,
        actor_id: *actor_id, // We have the actor id by this point, since Setup is done earlier
        account_id: 0,       // TODO: fill as we need it
        content_id: 0,
    };
    let _ = my_send.send(handle);
}

/// The task that kickstarts the client chat connection loop.
async fn start_chat_connection(my_handle: oneshot::Receiver<ClientHandle>, data: ClientChatData) {
    // Recieve client information from global
    let my_handle = match my_handle.await {
        Ok(my_handle) => my_handle,
        Err(_) => return,
    };

    let connection = data.connection;
    let recv = data.recv;

    // communication channel between client_loop and client_server_loop
    let (internal_send, internal_recv) = unbounded_channel();

    let _ = join!(
        tokio::spawn(client_chat_loop(connection, internal_recv, my_handle)),
        // TODO: It should be okay to reuse the zone connection's client_server_loop for ToServer & FromServer stuff since it gives us our own comms channel, right?
        tokio::spawn(client_server_loop(recv, internal_send))
    );
}

/// The client's chat connection loop, which allows them to communicate with other players in channels that span across the entire world.
async fn client_chat_loop(
    mut connection: ChatConnection,
    mut internal_recv: UnboundedReceiver<FromServer>,
    client_handle: ClientHandle,
) {
    // tell the server we exist, now that we confirmed we are a legitimate connection
    connection
        .handle
        .send(ToServer::NewChatClient(client_handle.clone()))
        .await;

    let mut buf = vec![0; RECEIVE_BUFFER_SIZE];
    loop {
        tokio::select! {
            biased; // client data should always be prioritized

            n = connection.socket.read(&mut buf) => {
                match n {
                    Ok(n) => {
                        // if the last response was over >5 seconds, the client is probably gone
                        if n == 0 {
                            let now = Instant::now();
                            if now.duration_since(connection.last_keep_alive) > NETWORK_TIMEOUT {
                                tracing::info!("Dropping chat connection from {:?} because it timed out", client_handle.id);
                                break;
                            }
                        } else {
                            connection.last_keep_alive = Instant::now();
                            let segments = connection.parse_packet(&buf);
                            for segment in segments {
                                match &segment.data {
                                    SegmentData::None() => {}
                                    SegmentData::Setup { .. } => {
                                        // Handled before our connection was spawned!
                                    }
                                    SegmentData::Ipc(data) => {
                                        match &data.data {
                                            ClientChatIpcData::SendTellMessage(tell_data) => {
                                                connection.send_tell_message(tell_data).await;
                                            }
                                            ClientChatIpcData::SendPartyMessage(data) => {
                                                connection.send_party_message(data).await;
                                            }
                                            ClientChatIpcData::GetChannelList { unk } => {
                                                tracing::info!("GetChannelList: {:#?} from {}", unk, connection.player_data.actor_id);
                                            }
                                            ClientChatIpcData::SendCWLinkshellMessage(data) => {
                                                connection.send_linkshell_message(data).await;
                                            }
                                            ClientChatIpcData::SendAllianceMessage(_data) => {
                                                tracing::info!("Chatting in alliances is unimplemented");
                                            }
                                            ClientChatIpcData::DiceRollCWLS(data) | ClientChatIpcData::DiceRollParty(data) => {
                                                connection.send_dice_roll(*data).await;
                                            }
                                            ClientChatIpcData::Unknown { unk } => {
                                                tracing::warn!("Unknown chat packet {:?} recieved ({} bytes)", data.header.op_code, unk.len());
                                                kawari_world::fork_opcodes::hint_unknown_packet("ClientChatIpcType", kawari::packet::PredefinedOpcode::get_opcode(&data.header.op_code), unk.len());
                                            }
                                        }
                                    }
                                    SegmentData::KeepAliveRequest { id, timestamp } => connection.send_keep_alive(*id, *timestamp).await,
                                    SegmentData::KeepAliveResponse { .. } => {
                                        // these should be safe to ignore
                                    }
                                    _ => panic!("ChatConnection: The server is receiving a response or an unknown packet: {segment:#?}"),
                                }
                            }
                        }
                    }
                    Err(_) => {
                        tracing::info!("ChatConnection {:#?} was killed because of a network error!", client_handle.id);
                        break;
                    },
                }
            }

            msg = internal_recv.recv() => match msg {
                Some(msg) => match msg {
                    FromServer::TellMessageReceived(from_actor_id, message_info) => connection.tell_message_received(from_actor_id, message_info.clone()).await,
                    FromServer::BotTellNotFound(name) => connection.send_tell_not_found(0, name).await,
                    FromServer::ChatDisconnected => {
                        tracing::info!("ChatConnection {:#?} received shutdown, disconnecting!", connection.id);
                        break;
                    }
                    FromServer::SetPartyChatChannel(channel_id) => connection.set_party_chatchannel(channel_id).await,
                    FromServer::PartyMessageReceived(message_data) => connection.party_message_received(message_data).await,
                    FromServer::MustRefreshChatChannels => connection.refresh_chatchannels().await,
                    FromServer::CWLSMessageReceived(message_info) => connection.cwls_message_received(message_info).await,
                    FromServer::ChatDiceRoll(from_actor_id, roll_data) => connection.dice_roll_received(from_actor_id, roll_data).await,
                    _ => tracing::error!("ChatConnection {:#?} received a FromServer message we don't care about: {:#?}, ensure you're using the right client network or that you've implemented a handler for it if we actually care about it!", client_handle.id, msg),
                },
                None => break,
            }
        }
    }

    connection
        .handle
        .send(ToServer::ChatDisconnected(connection.id))
        .await;
}

struct ClientZoneData {
    /// Socket for data recieved from the global server
    recv: Receiver<FromServer>,
    connection: ZoneConnection,
}

/// Spawn a new client actor.
fn spawn_client(connection: ZoneConnection) {
    let (send, recv) = channel(CLIENT_QUEUE_SIZE);

    let id = &connection.id.clone();
    let actor_id = &connection.player_data.character.actor_id.clone();
    let content_id = &connection.player_data.character.content_id.clone();
    let account_id = &connection.player_data.character.service_account_id.clone();

    let data = ClientZoneData { recv, connection };

    // Spawn a new client task
    let (my_send, my_recv) = oneshot::channel();
    let _kill = tokio::spawn(start_client(my_recv, data));

    // Send client information to said task
    let handle = ClientHandle {
        id: *id,
        channel: send,
        actor_id: *actor_id, // We have the actor id by this point, since Setup is done earlier
        content_id: *content_id as u64,
        account_id: *account_id as u64,
    };
    let _ = my_send.send(handle);
}

async fn start_client(my_handle: oneshot::Receiver<ClientHandle>, data: ClientZoneData) {
    // Recieve client information from global
    let my_handle = match my_handle.await {
        Ok(my_handle) => my_handle,
        Err(_) => return,
    };

    let connection = data.connection;
    let recv = data.recv;

    // communication channel between client_loop and client_server_loop
    let (internal_send, internal_recv) = unbounded_channel();

    let _ = join!(
        tokio::spawn(client_loop(connection, internal_recv, my_handle)),
        tokio::spawn(client_server_loop(recv, internal_send))
    );
}

async fn client_server_loop(
    mut data: Receiver<FromServer>,
    internal_send: UnboundedSender<FromServer>,
) {
    while let Some(msg) = data.recv().await {
        let Ok(_) = internal_send.send(msg) else {
            // In most cases this should be fine, as it's simply the zone and chat connection loops ending
            // If needed we can log messages here for troubleshooting, but previously this was just an unwrap which assumed it'd never fail
            break;
        };
    }
}

/// Process packets from the client. Returns false if we want to kill the connection.
async fn process_packet(
    connection: &mut ZoneConnection,
    lua_player: &mut LuaPlayer,
    events: &mut Vec<(Box<dyn EventHandler>, Event)>,
    client_handle: ClientHandle,
    n: usize,
    buf: &[u8],
) -> bool {
    // if the last response was over >5 seconds, the client is probably gone
    if n == 0 {
        let now = Instant::now();
        if now.duration_since(connection.last_keep_alive) > NETWORK_TIMEOUT {
            tracing::info!(
                "Dropping zone connection from {:?} because it timed out",
                client_handle.id
            );
            return false;
        }
    } else {
        if connection.hide_spectator_ui {
            connection
                .actor_control_self(ActorControlCategory::HideSpectatorUI {})
                .await;
            connection.hide_spectator_ui = false;
        }

        connection.last_keep_alive = Instant::now();

        let segments = connection.parse_packet(&buf[..n]);
        for segment in &segments {
            match &segment.data {
                SegmentData::None() => {}
                SegmentData::Setup { .. } => {
                    // Handled before our connection was spawned!
                }
                SegmentData::Ipc(data) => {
                    match &data.data {
                        ClientZoneIpcData::InitRequest { .. } => {
                            // IPC Init(?)
                            {
                                let ipc =
                                    ServerZoneIpcSegment::new(ServerZoneIpcData::InitResponse {
                                        actor_id: connection.player_data.character.actor_id,
                                    });
                                connection.send_ipc_self(ipc).await;
                            }

                            let service_account_id;
                            {
                                let mut database = connection.database.lock();
                                service_account_id = database.find_service_account(
                                    connection.player_data.character.content_id as u64,
                                );
                            }

                            let config = get_config();
                            let Ok(mut login_reply) = ureq::get(format!(
                                "{}/_private/max_ex?service={}",
                                config.login.server_name, service_account_id,
                            ))
                            .call() else {
                                tracing::warn!(
                                    "Failed to find service account {service_account_id}, just going to stop talking to this connection..."
                                );
                                return false;
                            };

                            let expansion: BasicServiceAccountData = serde_json::from_str(
                                &login_reply.body_mut().read_to_string().unwrap(),
                            )
                            .unwrap();
                            // Send inventory
                            connection.send_inventory().await;

                            connection.ensure_valid_zone().await;

                            // set equip display flags
                            connection
                                .actor_control_self(ActorControlCategory::SetEquipDisplayFlags {
                                    display_flag: connection.player_data.volatile.display_flags,
                                })
                                .await;

                            // Store when we logged in, for various purposes.
                            connection.login_time = Some(SystemTime::now());
                            connection.playtime_at_login = connection
                                .database
                                .lock()
                                .find_playtime(connection.player_data.character.content_id as u64);

                            // Mark the player as online for total player counts, player searches, etc.
                            {
                                connection.player_data.volatile.is_online = true;
                                let mut database = connection.database.lock();
                                database.commit_volatile(&connection.player_data);
                            }

                            // Characters who attuned to a second aetheryte before this was handled get Teleport now.
                            connection.check_teleport_unlock(false).await;
                            // Likewise what their finished quests unlock (e.g. Recommendations).
                            connection.check_quest_unlocks().await;
                            // And the city aethernets' extra destinations for those with all its shards already.
                            connection.learn_aethernet_extras().await;

                            // Stats
                            connection.send_stats().await;

                            // As seen in retail, they pad it with the first value
                            let mut padded_exp = connection.player_data.classjob.exp.0.clone();
                            padded_exp.resize(CLASSJOB_ARRAY_SIZE, padded_exp[0]);

                            // Ditto for levels
                            let mut padded_levels: Vec<u16> =
                                connection.player_data.classjob.levels.0.to_vec();
                            padded_levels.resize(CLASSJOB_ARRAY_SIZE, padded_levels[0]);

                            // Player Setup
                            {
                                let mut player_state_flags1 = PlayerStateFlags1::empty();
                                if connection.player_data.mentor.is_novice == 0 {
                                    player_state_flags1.set(PlayerStateFlags1::NOT_NOVICE, true);
                                }
                                if connection.player_data.mentor.is_battle == 1 {
                                    player_state_flags1.set(PlayerStateFlags1::BATTLE_MENTOR, true);
                                }

                                let mut player_state_flags2 = PlayerStateFlags2::empty();
                                if connection.player_data.mentor.is_returner == 1 {
                                    player_state_flags2.set(PlayerStateFlags2::RETURNER, true);
                                }

                                let mut player_state_flags3 = PlayerStateFlags3::empty();
                                if connection.player_data.mentor.is_trade == 1 {
                                    player_state_flags3.set(PlayerStateFlags3::TRADE_MENTOR, true);
                                }

                                let mut quest_special_flags = QuestSpecialFlags::empty();
                                if connection.player_data.character.warrior_of_light == 1 {
                                    quest_special_flags.insert(QuestSpecialFlags::WARRIOR_OF_LIGHT);
                                }
                                if expansion.legacy {
                                    quest_special_flags.insert(QuestSpecialFlags::LEGACY);
                                }

                                let ipc = ServerZoneIpcSegment::new(
                                    ServerZoneIpcData::PlayerSetup(PlayerSetup {
                                        content_id: connection.player_data.character.content_id
                                            as u64,
                                        player_state_flags1,
                                        player_state_flags2,
                                        player_state_flags3,
                                        exp: padded_exp,
                                        max_level: calculate_max_level(expansion.max_ex as u8),
                                        expansion: expansion.max_ex as u8,
                                        name: connection.player_data.character.name.clone(),
                                        actor_id: connection.player_data.character.actor_id,
                                        race: connection
                                            .player_data
                                            .customize
                                            .chara_make
                                            .customize
                                            .race
                                            as u8,
                                        sex: connection
                                            .player_data
                                            .customize
                                            .chara_make
                                            .customize
                                            .gender
                                            as u8,
                                        tribe: connection
                                            .player_data
                                            .customize
                                            .chara_make
                                            .customize
                                            .tribe
                                            as u8,
                                        start_town: connection.player_data.customize.city_state
                                            as u8,
                                        birth_month: connection
                                            .player_data
                                            .customize
                                            .chara_make
                                            .birth_month
                                            as u8,
                                        birth_day: connection
                                            .player_data
                                            .customize
                                            .chara_make
                                            .birth_day
                                            as u8,
                                        guardian_deity: connection
                                            .player_data
                                            .customize
                                            .chara_make
                                            .guardian
                                            as u8,
                                        current_classjob_id: connection
                                            .player_data
                                            .classjob
                                            .current_class
                                            as u8,
                                        first_class: connection.player_data.classjob.first_class
                                            as u8,
                                        levels: padded_levels,
                                        unlocks: connection.player_data.unlock.unlocks.data.clone(),
                                        aetherytes: connection
                                            .player_data
                                            .aetheryte
                                            .unlocked
                                            .data
                                            .clone(),
                                        minions: connection.player_data.unlock.minions.data.clone(),
                                        mounts: connection.player_data.unlock.mounts.data.clone(),
                                        home_aetheryte_id: connection
                                            .player_data
                                            .aetheryte
                                            .homepoint
                                            as u16,
                                        favourite_aetheryte_count: 1,
                                        favorite_aetheryte_ids: [8, 0, 0, 0],
                                        seen_active_help: connection
                                            .player_data
                                            .unlock
                                            .seen_active_help
                                            .data_truncated(),
                                        aether_currents_mask: connection
                                            .player_data
                                            .aether_current
                                            .unlocked
                                            .data_truncated(),
                                        orchestrion_roll_mask: connection
                                            .player_data
                                            .unlock
                                            .orchestrion_rolls
                                            .data_truncated(),
                                        buddy_equip_mask: connection
                                            .player_data
                                            .companion
                                            .unlocked_equip
                                            .data_truncated(),
                                        cutscene_seen_mask: connection
                                            .player_data
                                            .unlock
                                            .cutscene_seen
                                            .data_truncated(),
                                        ornament_mask: connection
                                            .player_data
                                            .unlock
                                            .ornaments
                                            .data_truncated(),
                                        caught_fish_mask: connection
                                            .player_data
                                            .unlock
                                            .caught_fish
                                            .data_truncated(),
                                        caught_spearfish_mask: connection
                                            .player_data
                                            .unlock
                                            .caught_spearfish
                                            .data_truncated(),
                                        adventure_mask: connection
                                            .player_data
                                            .unlock
                                            .adventures
                                            .data_truncated(),
                                        triple_triad_cards: connection
                                            .player_data
                                            .unlock
                                            .triple_triad_cards
                                            .data_truncated(),
                                        glasses_styles_mask: connection
                                            .player_data
                                            .unlock
                                            .glasses_styles
                                            .data_truncated(),
                                        chocobo_taxi_stands_mask: connection
                                            .player_data
                                            .unlock
                                            .chocobo_taxi_stands
                                            .data_truncated(),
                                        maps_with_up_to_16_regions: map_discovery_u16(
                                            &connection.player_data.unlock.map_discovery.data,
                                        ),
                                        maps_with_up_to_32_regions: map_discovery_u32(
                                            &connection.player_data.unlock.map_discovery.data,
                                        ),
                                        aether_current_comp_flg_set_bitmask1: connection
                                            .player_data
                                            .aether_current
                                            .comp_flg_set
                                            .data[0],
                                        aether_current_comp_flg_set_bitmask2: connection
                                            .player_data
                                            .aether_current
                                            .comp_flg_set
                                            .data[1..AETHER_CURRENT_COMP_FLG_SET_BITMASK_SIZE]
                                            .to_vec(),

                                        // content
                                        unlocked_special_content: connection
                                            .player_data
                                            .content
                                            .unlocked_special_content
                                            .data_truncated(),
                                        unlocked_dungeons: connection
                                            .player_data
                                            .content
                                            .unlocked_dungeons
                                            .data_truncated(),
                                        unlocked_raids: connection
                                            .player_data
                                            .content
                                            .unlocked_raids
                                            .data_truncated(),
                                        unlocked_guildhests: connection
                                            .player_data
                                            .content
                                            .unlocked_guildhests
                                            .data_truncated(),
                                        unlocked_trials: connection
                                            .player_data
                                            .content
                                            .unlocked_trials
                                            .data_truncated(),
                                        unlocked_crystalline_conflict: connection
                                            .player_data
                                            .content
                                            .unlocked_crystalline_conflicts
                                            .data_truncated(),
                                        unlocked_frontline: connection
                                            .player_data
                                            .content
                                            .unlocked_frontlines
                                            .data_truncated(),
                                        cleared_raids: connection
                                            .player_data
                                            .content
                                            .cleared_raids
                                            .data_truncated(),
                                        cleared_dungeons: connection
                                            .player_data
                                            .content
                                            .cleared_dungeons
                                            .data_truncated(),
                                        cleared_guildhests: connection
                                            .player_data
                                            .content
                                            .cleared_guildhests
                                            .data_truncated(),
                                        cleared_trials: connection
                                            .player_data
                                            .content
                                            .cleared_trials
                                            .data_truncated(),
                                        cleared_crystalline_conflict: connection
                                            .player_data
                                            .content
                                            .cleared_crystalline_conflicts
                                            .data_truncated(),
                                        cleared_frontline: connection
                                            .player_data
                                            .content
                                            .cleared_frontlines
                                            .data_truncated(),
                                        cleared_masked_carnivale: connection
                                            .player_data
                                            .content
                                            .cleared_masked_carnivale
                                            .data_truncated(),
                                        unlocked_misc_content: connection
                                            .player_data
                                            .content
                                            .unlocked_misc_content
                                            .data_truncated(),
                                        cleared_misc_content: connection
                                            .player_data
                                            .content
                                            .cleared_misc_content
                                            .data_truncated(),
                                        can_do_triple_triad_matches: true,
                                        ui_festival_ids: connection
                                            .config
                                            .active_festivals
                                            .map(FestivalId),
                                        quest_special_flags,
                                        companion_name: connection.player_data.buddy.name.clone(),
                                        buddy_levels: connection
                                            .player_data
                                            .buddy
                                            .levels
                                            .0
                                            .clone()
                                            .try_into()
                                            .unwrap(),
                                        companion_current_exp: connection.player_data.buddy.exp
                                            as u32,
                                        companion_rank: connection.player_data.buddy.rank as u8,
                                        companion_stars: connection.player_data.buddy.stars as u8,
                                        companion_color: connection.player_data.buddy.color as u8,
                                        companion_skill_points: connection
                                            .player_data
                                            .buddy
                                            .skill_points
                                            as u8,
                                        companion_active_command: connection
                                            .player_data
                                            .buddy
                                            .active_command
                                            as u8,
                                        companion_favorite_feed: connection
                                            .player_data
                                            .buddy
                                            .favorite_feed
                                            as u8,
                                        buddy_equip_row_ids: connection.companion_profile().barding,
                                        completed_lovm_stages: 255, // TODO: placeholder to show them in the Duty Finder
                                        ..Default::default()
                                    }),
                                );
                                connection.send_ipc_self(ipc).await;
                            }

                            let level;
                            {
                                let mut game_data = connection.game_data.lock();

                                level = connection
                                    .player_data
                                    .inventory
                                    .equipped
                                    .calculate_item_level(&mut game_data)
                                    as u32;
                            }

                            connection
                                .actor_control_self(ActorControlCategory::SetItemLevel { level })
                                .await;
                            // The chocobo's colour, as retail sent it at every login (see `CompanionColor`.)
                            let stain = connection.companion_profile().stain() as u32;
                            connection
                                .actor_control_self(ActorControlCategory::CompanionColor { stain })
                                .await;

                            connection
                                .handle
                                .send(ToServer::ReadySpawnPlayer(
                                    connection.id,
                                    connection.player_data.character.actor_id,
                                    connection.player_data.volatile.zone_id as u16,
                                    connection.player_data.volatile.position,
                                    connection.player_data.volatile.rotation as f32,
                                    if connection.player_data.unlock.cutscene_seen.contains(2) {
                                        None
                                    } else {
                                        Some(connection.player_data.city_state)
                                    }, // If seen the opening cutscene
                                ))
                                .await;

                            connection.send_mailbox_status().await;
                            connection.init_linkshells().await;
                            connection.send_crossworld_linkshells(false).await;
                            connection.send_grand_company_info().await;

                            // Send login message
                            let config = get_config();
                            connection.send_notice(&config.world.login_message).await;

                            let online_player_count;
                            {
                                let mut db = connection.database.lock();
                                online_player_count = db.get_online_player_count();
                            }

                            connection
                                .send_notice(&format!(
                                    "There are currently {} players online.",
                                    online_player_count
                                ))
                                .await;
                        }
                        ClientZoneIpcData::FinishLoading { .. } => {
                            let spawn = connection.respawn_player(true).await;

                            // tell the server we loaded into the zone, so it can start sending us actors
                            connection
                                .handle
                                .send(ToServer::ZoneLoaded(
                                    connection.id,
                                    connection.player_data.character.actor_id,
                                    spawn.clone(),
                                ))
                                .await;

                            // If we're in a party, we need to tell the other members we changed areas or reconnected.
                            if connection.is_in_party() {
                                if !connection.rejoining_party {
                                    connection
                                        .handle
                                        .send(ToServer::PartyMemberChangedAreas(
                                            connection.party_id,
                                            connection.player_data.character.service_account_id
                                                as u64,
                                            connection.player_data.character.content_id as u64,
                                            connection.player_data.character.actor_id,
                                            connection.player_data.character.name.clone(),
                                            connection.player_data.volatile.zone_id,
                                        ))
                                        .await;
                                } else {
                                    connection
                                        .handle
                                        .send(ToServer::PartyMemberReturned(
                                            connection.player_data.character.actor_id,
                                            connection.player_data.volatile.zone_id,
                                            connection.party_id,
                                        ))
                                        .await;
                                    connection.rejoining_party = false;
                                }
                            }

                            connection.send_stats().await;
                        }
                        ClientZoneIpcData::ClientTrigger(trigger) => {
                            match trigger.trigger {
                                ClientTriggerCommand::RequestTitleList => {
                                    let ipc =
                                        ServerZoneIpcSegment::new(ServerZoneIpcData::TitleList {
                                            unlock_bitmask: connection
                                                .player_data
                                                .unlock
                                                .titles
                                                .data
                                                .clone(),
                                        });
                                    connection.send_ipc_self(ipc).await;
                                }
                                ClientTriggerCommand::FinishZoning => {
                                    // The server needs the chocobo companion to bring it along or say it isn't out.
                                    connection.send_companion().await;
                                    connection
                                        .handle
                                        .send(ToServer::ZoneIn(
                                            connection.id,
                                            connection.player_data.character.actor_id,
                                            connection.teleport_reason == TeleportReason::Aetheryte,
                                        ))
                                        .await;

                                    // Reset so it doesn't get stuck to Aetheryte:
                                    connection.teleport_reason = TeleportReason::NotSpecified;
                                }
                                ClientTriggerCommand::BeginContentsReplay => {
                                    connection.conditions.set_condition(
                                        kawari::ipc::zone::Condition::DutyRecorderPlayback,
                                    );
                                    connection.send_conditions().await;

                                    connection
                                        .actor_control_self(
                                            ActorControlCategory::BeginContentsReplay { unk1: 1 },
                                        )
                                        .await;
                                }
                                ClientTriggerCommand::EndContentsReplay => {
                                    connection.conditions.remove_condition(
                                        kawari::ipc::zone::Condition::DutyRecorderPlayback,
                                    );
                                    connection.send_conditions().await;

                                    connection
                                        .actor_control_self(
                                            ActorControlCategory::EndContentsReplay { unk1: 1 },
                                        )
                                        .await;

                                    connection.respawn_player(false).await;
                                }
                                ClientTriggerCommand::Dismount { sequence } => {
                                    // TODO: Move all this to FromServer::ActorDismounted so all of the logic can be consolidated
                                    connection.conditions = Conditions::default();
                                    connection.send_conditions().await;

                                    // SetMode isn't important, no, but it's included for accuracy.
                                    connection
                                        .set_character_mode(CharacterMode::Normal, 0, true)
                                        .await;

                                    // Retail indeed does send an AC, not an ACS for this, with where the player lands.
                                    let (unk1, unk2, unk3) = kawari::common::dismount_landing(
                                        &connection.player_data.volatile.position,
                                        connection.player_data.volatile.rotation as f32,
                                    );
                                    connection
                                        .actor_control(
                                            connection.player_data.character.actor_id,
                                            ActorControlCategory::PlayDismountAnimation {
                                                unk1,
                                                unk2,
                                                unk3,
                                            },
                                        )
                                        .await;

                                    // TODO: This should only be sent when the player is actually riding pillion, but for now, it doesn't seem to hurt sending it unconditionally.
                                    connection
                                        .actor_control(
                                            connection.player_data.character.actor_id,
                                            ActorControlCategory::RidePillion {
                                                target_actor_id: ObjectId::default(),
                                                target_seat_index: 0,
                                            },
                                        )
                                        .await;

                                    connection
                                        .actor_control_self(ActorControlCategory::Dismount {
                                            sequence,
                                        })
                                        .await;

                                    // TODO: Remove this `let party_id` in an upcoming party refactor, this is temporary
                                    let party_id = if connection.party_id != 0 {
                                        Some(connection.party_id)
                                    } else {
                                        None
                                    };
                                    connection
                                        .handle
                                        .send(ToServer::Dismounted(
                                            connection.player_data.character.actor_id,
                                            party_id,
                                        ))
                                        .await;
                                }
                                // The chocobo companion's menu and window, and its saddlebag (zone_connection/companion.rs.)
                                ClientTriggerCommand::WithdrawCompanion { unk1 } => {
                                    connection.companion_command(unk1).await;
                                }
                                ClientTriggerCommand::SetCompanionBarding { slot, equip } => {
                                    connection.set_companion_barding(slot, equip).await;
                                }
                                ClientTriggerCommand::LearnCompanionSkill { line } => {
                                    connection.learn_companion_skill(line).await;
                                }
                                ClientTriggerCommand::OpenChocoboSaddlebag => {
                                    connection.open_saddlebag().await;
                                }
                                ClientTriggerCommand::ShownActiveHelp { id } => {
                                    // Save this so it isn't shown again on next login
                                    connection.player_data.unlock.seen_active_help.set(id);
                                }
                                ClientTriggerCommand::SeenCutscene { id } => {
                                    connection.player_data.unlock.cutscene_seen.set(id);
                                }
                                // The Mini Cactpot's scratches go to its event (`events/gold_saucer.rs`.)
                                ClientTriggerCommand::DirectorTrigger {
                                    handler_id,
                                    trigger: director_trigger,
                                } if events
                                    .last()
                                    .is_some_and(|event| event.1.id == handler_id) =>
                                {
                                    if let Some(event) = events.last() {
                                        event
                                            .0
                                            .on_trigger(
                                                &event.1,
                                                connection,
                                                &director_trigger,
                                                lua_player,
                                            )
                                            .await;
                                    }
                                }
                                // The client asking its GATE director what's on (`zone_connection/gate.rs`.)
                                ClientTriggerCommand::DirectorTrigger {
                                    handler_id,
                                    trigger: director_trigger,
                                } if handler_id.handler_type() == HandlerType::GoldSaucer => {
                                    connection.gate_trigger(handler_id, &director_trigger).await;
                                }
                                ClientTriggerCommand::OpenGoldSaucerGeneralTab => {
                                    let ipc = ServerZoneIpcSegment::new(
                                        ServerZoneIpcData::GoldSaucerInformation { unk: [0; 40] },
                                    );
                                    connection.send_ipc_self(ipc).await;
                                }
                                ClientTriggerCommand::OpenTrustWindow => {
                                    // We have to send at least one valid trust to the client, otherwise the window never shows.
                                    let ipc = ServerZoneIpcSegment::new(
                                        ServerZoneIpcData::TrustInformation(TrustInformation {
                                            available_content: vec![TrustContent {
                                                trust_content_id: 1, // Holminster Switch
                                                last_selected_characters: [0xFF; 16],
                                            }],
                                            levels: [0; TrustInformation::NUM_CHARACTERS],
                                            exp: [0; TrustInformation::NUM_CHARACTERS],
                                        }),
                                    );
                                    connection.send_ipc_self(ipc).await;
                                }
                                // A duty's entrance asking for its Duty Support window: retail answered with the
                                // duty (DawnContent) and 0, and the window opened (Bob's capture of 2026-10-23.)
                                ClientTriggerCommand::OpenDutySupportFor { dawn_content_id } => {
                                    let ipc = ServerZoneIpcSegment::new(
                                        ServerZoneIpcData::DutySupportContent {
                                            dawn_content_id,
                                            unk: 0,
                                        },
                                    );
                                    connection.send_ipc_self(ipc).await;
                                }
                                ClientTriggerCommand::OpenDutySupportWindow => {
                                    let mut available_content =
                                        Vec::with_capacity(DutySupportInformation::INDICE_COUNT);
                                    {
                                        let mut game_data = connection.game_data.lock();
                                        let content_finder_conditions =
                                            game_data.list_dawn_content_content_finder_conditions();

                                        for (i, cfc) in content_finder_conditions {
                                            let instance_id = game_data
                                                .find_content_for_content_finder_id(cfc as u16)
                                                .unwrap();
                                            let content_type = game_data
                                                .find_type_for_content(instance_id)
                                                .unwrap();
                                            let unlocked = match content_type {
                                                InstanceContentType::Dungeon => connection
                                                    .player_data
                                                    .content
                                                    .unlocked_dungeons
                                                    .contains(instance_id as u32 - 1),
                                                InstanceContentType::Raid => connection
                                                    .player_data
                                                    .content
                                                    .unlocked_raids
                                                    .contains(instance_id as u32 - 30001),
                                                _ => false,
                                            };
                                            if unlocked {
                                                available_content.push(i as u8);
                                            }
                                        }
                                    }

                                    // We have to send at least one available duty to the client, otherwise it crashes.
                                    let ipc = ServerZoneIpcSegment::new(
                                        ServerZoneIpcData::DutySupportInformation(
                                            DutySupportInformation { available_content },
                                        ),
                                    );
                                    connection.send_ipc_self(ipc).await;
                                }
                                ClientTriggerCommand::OpenPortraitsWindow => {
                                    // The adventurer plate's portrait as the client last set it (all zeroes if never.)
                                    let ipc = ServerZoneIpcSegment::new(
                                        ServerZoneIpcData::PortraitsInformation {
                                            portrait: connection
                                                .player_data
                                                .volatile
                                                .portrait
                                                .data_truncated(),
                                        },
                                    );
                                    connection.send_ipc_self(ipc).await;
                                }
                                ClientTriggerCommand::RequestAchievements => {
                                    connection.send_achievements().await;
                                }
                                ClientTriggerCommand::RequestAchievementProgress { id } => {
                                    connection.send_achievement_progress(id).await;
                                }
                                ClientTriggerCommand::RequestNearCompletionAchievements {
                                    slots,
                                } => {
                                    // Nearly completed ones aren't worked out yet, so each list comes back empty.
                                    for slot in 0..2 {
                                        if slots & (1 << slot) != 0 {
                                            connection
                                                .actor_control_self(
                                                    ActorControlCategory::UnkAchievementRelated {
                                                        unk1: slot,
                                                        unk2: 0,
                                                        unk3: 0,
                                                        unk4: 0,
                                                    },
                                                )
                                                .await;
                                        }
                                    }
                                }
                                ClientTriggerCommand::SetTitle { title_id } => {
                                    connection.player_data.volatile.title = title_id as i32;

                                    // Inform the server, so it sends out the AC.
                                    connection
                                        .handle
                                        .send(ToServer::ClientTrigger(
                                            connection.id,
                                            connection.player_data.character.actor_id,
                                            trigger.clone(),
                                        ))
                                        .await;
                                }
                                ClientTriggerCommand::AbandonContent { .. } => {
                                    // Leaving from a scene of the duty's director (Air Force One's result): retail
                                    // finished its event first (Bob's Chronofoil captures deaf7e28 and 2c98f9e8.)
                                    if events.last().is_some_and(|event| {
                                        Some(event.1.id) == connection.content_handler_id
                                    }) {
                                        connection.event_finish(events).await;
                                    }
                                    // Remove ourselves from this instance.
                                    connection
                                        .handle
                                        .send(ToServer::LeaveContent(
                                            connection.id,
                                            connection.player_data.character.actor_id,
                                            connection.old_zone_id,
                                            connection.old_position,
                                            connection.old_rotation,
                                        ))
                                        .await;
                                }
                                ClientTriggerCommand::PrepareCastGlamour { .. } => {
                                    // The actual glamoruing happens later when the action is complete.
                                    connection.glamour_information = Some(trigger.trigger.clone());
                                }
                                ClientTriggerCommand::PrepareRemoveGlamour { .. } => {
                                    // Ditto.
                                    connection.glamour_information = Some(trigger.trigger.clone());
                                }
                                ClientTriggerCommand::BeginOrEndFishing { end } => {
                                    let handler_id = HandlerId::new(HandlerType::Fishing, 1);
                                    if !end {
                                        connection
                                            .start_event(
                                                ObjectTypeId {
                                                    object_id: connection
                                                        .player_data
                                                        .character
                                                        .actor_id,
                                                    object_type: ObjectTypeKind::None,
                                                },
                                                handler_id,
                                                EventType::Fishing,
                                                0,
                                                events,
                                                lua_player,
                                            )
                                            .await;

                                        let event = &events.last().unwrap().1;

                                        // TODO: wrong scene flags
                                        connection
                                            .event_scene(
                                                event,
                                                1,
                                                SceneFlags::NO_DEFAULT_CAMERA,
                                                vec![274, 277, 0],
                                            )
                                            .await;

                                        let ipc = EventLogMessage {
                                            handler_id,
                                            message_type: LogMessageType::FishingCastLine as u32,
                                            params: vec![28],
                                        }
                                        .package()
                                        .unwrap();
                                        connection.send_ipc_self(ipc).await;

                                        connection
                                            .handle
                                            .send(ToServer::Fish(
                                                connection.id,
                                                connection.player_data.character.actor_id,
                                            ))
                                            .await;
                                    } else {
                                        let event = &events.last().unwrap().1;

                                        connection
                                            .event_scene(
                                                event,
                                                3,
                                                SceneFlags::NO_DEFAULT_CAMERA,
                                                vec![273],
                                            )
                                            .await;
                                    }
                                }
                                ClientTriggerCommand::RequestGatheringPoint { id } => {
                                    let base_id;
                                    let level;
                                    let count;
                                    let gathering_point_bonus;
                                    let point_type;
                                    {
                                        let mut game_data = connection.game_data.lock();
                                        (
                                            base_id,
                                            level,
                                            count,
                                            gathering_point_bonus,
                                            point_type,
                                        ) = game_data.get_gathering_point(id);
                                    }

                                    // TODO: document in EXDSchema that 3 seems to mean levequest ones
                                    if point_type != 3 {
                                        let remaining = connection.gathering.remaining(id, count);
                                        connection
                                            .actor_control_self(
                                                ActorControlCategory::SetupGatheringPoint {
                                                    id,
                                                    base_id: base_id as u32,
                                                    level: level as u32,
                                                    count: count as u32,
                                                    remaining_count: remaining as u32,
                                                    unk1: 0,
                                                    gathering_point_bonus,
                                                },
                                            )
                                            .await;
                                    }
                                }
                                ClientTriggerCommand::BeginCraft { end, id } => {
                                    let handler_id = HandlerId::new(HandlerType::Craft, 1);
                                    if !end {
                                        connection
                                            .start_event(
                                                ObjectTypeId {
                                                    object_id: connection
                                                        .player_data
                                                        .character
                                                        .actor_id,
                                                    object_type: ObjectTypeKind::None,
                                                },
                                                handler_id,
                                                EventType::Craft,
                                                0,
                                                events,
                                                lua_player,
                                            )
                                            .await;

                                        let event = &events.last().unwrap().1;

                                        let recipe;
                                        {
                                            let mut game_data = connection.game_data.lock();
                                            recipe = game_data.get_recipe(id);
                                        }

                                        connection.recipe = Some(recipe);
                                        connection.quick_synthesis = None;
                                        // Retail's start of a synthesis (Bob's capture, docs/src/fork/crafting.md.)
                                        kawari_world::begin_synthesis(connection, event, 0).await;
                                    }
                                }
                                ClientTriggerCommand::RidePillionRequest {
                                    target_actor_id,
                                    target_seat_index,
                                } => {
                                    // TODO: Remove this `let party_id` in an upcoming party refactor, this is temporary
                                    let party_id = if connection.party_id != 0 {
                                        Some(connection.party_id)
                                    } else {
                                        None
                                    };

                                    connection
                                        .handle
                                        .send(ToServer::RidePillionRequest(
                                            connection.player_data.character.actor_id,
                                            party_id,
                                            target_actor_id,
                                            target_seat_index,
                                        ))
                                        .await;
                                }
                                // Answered by the server, which has the one examined (`party_bot.rs`.)
                                ClientTriggerCommand::ExamineCharacter { target_actor_id } => {
                                    connection
                                        .handle
                                        .send(ToServer::ExamineCharacter(
                                            connection.player_data.character.actor_id,
                                            target_actor_id,
                                        ))
                                        .await;
                                }
                                ClientTriggerCommand::ToggleNoviceStatus => {
                                    if connection.player_data.search_info.online_status
                                        != OnlineStatus::NewAdventurer
                                    {
                                        connection.player_data.search_info.online_status =
                                            OnlineStatus::NewAdventurer;
                                    } else {
                                        connection.player_data.search_info.online_status =
                                            OnlineStatus::Online;
                                    }
                                    {
                                        let mut database = connection.database.lock();
                                        database.commit_search_info(&connection.player_data);
                                    }
                                    connection.update_online_status().await;
                                }
                                ClientTriggerCommand::OpenUnk1 { unk1, unk2, .. }
                                    if events.last().is_some_and(|x| {
                                        x.1.id.handler_type() == HandlerType::Quest
                                    }) =>
                                {
                                    // During a quest, this is a ServerRequestCallbackManager request: the quest battle
                                    // prompt (YesNoQuestBattle) asks with command 10 and waits for a reply, where 0
                                    // means it can be shown. unk1 is the callback slot and unk2 the command.
                                    // TODO: retail probably checks whether the player can enter, we always allow it.
                                    connection
                                        .actor_control_self(
                                            ActorControlCategory::ServerRequestReply {
                                                callback_index: unk1,
                                                command: unk2,
                                                result: 0,
                                            },
                                        )
                                        .await;
                                }
                                ClientTriggerCommand::OpenUnk1 {
                                    unk1,
                                    unk2,
                                    unk3,
                                    unk4,
                                } => {
                                    // At a summoning bell (docs/src/fork/retainers.md.)
                                    match unk2 {
                                        2 => connection.send_retainer_list().await,
                                        // The Mogpendium's window: retail answered with the Mogpendium packet, its
                                        // second and third words 0x18 and 0x20 (Bob's Chronofoil capture of
                                        // 2026-10-24). Nothing done this week.
                                        24 => {
                                            let ipc = ServerZoneIpcSegment::new(
                                                ServerZoneIpcData::Mogpendium(
                                                    kawari::ipc::zone::Mogpendium {
                                                        unk2: 0x18,
                                                        unk3: 0x20,
                                                        id_or_seed: connection
                                                            .player_data
                                                            .character
                                                            .content_id
                                                            as u32,
                                                        ..Default::default()
                                                    },
                                                ),
                                            );
                                            connection.send_ipc_self(ipc).await;
                                        }
                                        3 => {
                                            connection
                                                .send_one_retainer_info(retainer::join_id(
                                                    unk3, unk4,
                                                ))
                                                .await
                                        }
                                        command => {
                                            // Command 4 came as a retainer was called and again as their menu
                                            // reopened; retail answered with nothing but this.
                                            connection
                                                .actor_control_self(
                                                    ActorControlCategory::ServerRequestReply {
                                                        callback_index: unk1,
                                                        command,
                                                        result: 0,
                                                    },
                                                )
                                                .await;
                                        }
                                    }
                                }
                                ClientTriggerCommand::SetRetainerMarketPrice { slot, price } => {
                                    connection.set_retainer_market_price(slot, price);
                                }
                                // The glamour dresser, its plates and the armoire (docs/src/fork/glamour.md.)
                                ClientTriggerCommand::OpenGlamourDresser => {
                                    connection.send_glamour_dresser().await;
                                }
                                ClientTriggerCommand::ToggleGlamourPlatesWindow { .. } => {
                                    // Retail answered every 2356 with the plates; the client always sent 0 (the plate
                                    // list from the dresser or the character window, and before equipping a gear set
                                    // linked to a plate; Bob's captures of 2026-10-24.)
                                    connection.send_glamour_plates().await;
                                }
                                ClientTriggerCommand::ToggleGlamourPlateSelection { open } => {
                                    connection.toggle_glamour_plate_selection(open).await;
                                }
                                ClientTriggerCommand::ApplyGlamourPlate { plate } => {
                                    connection.apply_glamour_plate(plate as usize).await;
                                }
                                ClientTriggerCommand::RepairItem {
                                    container,
                                    slot,
                                    item_id,
                                } => {
                                    connection
                                        .repair_item(container, slot as u16, item_id)
                                        .await;
                                }
                                ClientTriggerCommand::Materialize {
                                    handler_id,
                                    param1,
                                    param2,
                                    param3,
                                } => {
                                    connection
                                        .begin_materialize(
                                            handler_id,
                                            [param1, param2, param3],
                                            events,
                                            lua_player,
                                        )
                                        .await;
                                }
                                ClientTriggerCommand::RepairArmouryGroup { group } => {
                                    connection.repair_armoury_group(group).await;
                                }
                                ClientTriggerCommand::RepairContainer { container } => {
                                    connection.repair_container(container).await;
                                }
                                // Doman Mahjong's tab: retail answered ActorControlSelf 2001 `[0, 1500, 1500]` for a
                                // character who hadn't played (Bob's capture of 2026-10-24), likely the starting rating.
                                ClientTriggerCommand::OpenGoldSaucerMahjongTab => {
                                    connection
                                        .actor_control_self(ActorControlCategory::Unknown {
                                            category: 2001,
                                            param1: 0,
                                            param2: 1500,
                                            param3: 1500,
                                            param4: 0,
                                            param5: 0,
                                        })
                                        .await;
                                }
                                ClientTriggerCommand::StoreInGlamourDresser { container, slot } => {
                                    connection
                                        .store_in_glamour_dresser(container, slot as u16)
                                        .await;
                                }
                                ClientTriggerCommand::RestoreFromGlamourDresser { index } => {
                                    connection.restore_from_glamour_dresser(index).await;
                                }
                                ClientTriggerCommand::OpenArmoire => {
                                    connection.send_armoire().await;
                                }
                                ClientTriggerCommand::OpenArmoireInfo => {
                                    connection.send_armoire_info().await;
                                }
                                ClientTriggerCommand::StoreInArmoire { cabinet_id } => {
                                    connection.store_in_armoire(cabinet_id).await;
                                }
                                ClientTriggerCommand::WithdrawFromArmoire { cabinet_id } => {
                                    connection.withdraw_from_armoire(cabinet_id).await;
                                }
                                // After every inventory change retail answered with an empty map marker update
                                // (Bob's Chronofoil capture of 2026-10-24.)
                                ClientTriggerCommand::InventoryChanged => {
                                    connection
                                        .actor_control_self(
                                            ActorControlCategory::MapMarkerUpdateBegin { flags: 1 },
                                        )
                                        .await;
                                    connection
                                        .actor_control_self(
                                            ActorControlCategory::MapMarkerUpdateEnd,
                                        )
                                        .await;
                                }
                                ClientTriggerCommand::RequestInventorySlot { container, slot } => {
                                    connection.send_inventory_slot(container, slot as u16).await;
                                }
                                ClientTriggerCommand::RequestApartmentList { starting_index } => {
                                    connection.send_apartment_list(starting_index).await;
                                }
                                ClientTriggerCommand::SetInteriorLightLevel { level, unk } => {
                                    // TODO: Also reject if they're not the owner/shared tenant
                                    let intended_use = lua_player.zone_data.intended_use;
                                    if !in_housing_area(intended_use) {
                                        continue;
                                    }

                                    connection
                                        .actor_control_self(
                                            ActorControlCategory::InteriorLightLevel {
                                                unk1: 0,
                                                level,
                                                unk2: unk,
                                            },
                                        )
                                        .await;

                                    connection
                                        .broadcast_actor_control(
                                            ActorControlCategory::InteriorLightLevelForObserver {
                                                level,
                                                unk1: unk,
                                            },
                                        )
                                        .await;
                                }
                                ClientTriggerCommand::FurnitureMenuToggled { closed } => {
                                    // TODO: Also reject if they're not the owner/shared tenant
                                    let intended_use = lua_player.zone_data.intended_use;
                                    if !in_housing_area(intended_use) {
                                        continue;
                                    }

                                    if !closed {
                                        connection.conditions.set_condition(
                                            kawari::ipc::zone::Condition::UsingHousingFunctions,
                                        );

                                        connection
                                            .actor_control_self(ActorControlCategory::FurnitureMenu)
                                            .await;
                                    } else {
                                        connection.conditions.remove_condition(
                                            kawari::ipc::zone::Condition::UsingHousingFunctions,
                                        );
                                    }
                                    connection.send_conditions().await;
                                }
                                ClientTriggerCommand::RequestHousingInventory { storeroom } => {
                                    // TODO: Also reject if they're not the owner/shared tenant
                                    let intended_use = lua_player.zone_data.intended_use;
                                    if !in_housing_area(intended_use) {
                                        continue;
                                    }

                                    let desired_pages = connection
                                        .player_data
                                        .house_inventory
                                        .get_desired_pages_from_intendeduse(
                                            intended_use,
                                            storeroom,
                                        );

                                    connection.send_housing_inventory(desired_pages).await;
                                }
                                ClientTriggerCommand::MoveHousingItemToInventory {
                                    to_storeroom,
                                    storage_id,
                                    slot,
                                    ..
                                } => {
                                    let intended_use = lua_player.zone_data.intended_use;
                                    // TODO: Also reject if they're not the owner/shared tenant
                                    if !in_housing_area(intended_use) {
                                        continue;
                                    }

                                    tracing::info!(
                                        "Client is moving housing item to inventory! {:#?} {:#?} {:#?}",
                                        to_storeroom,
                                        storage_id,
                                        slot
                                    );

                                    let desired_pages = connection
                                        .player_data
                                        .house_inventory
                                        .get_desired_pages_from_intendeduse(
                                            intended_use,
                                            to_storeroom,
                                        );

                                    let Some(transfer_item) = connection
                                        .player_data
                                        .house_inventory
                                        .get_item(storage_id, slot)
                                    else {
                                        continue;
                                    };

                                    let item_info = if !to_storeroom {
                                        connection
                                            .player_data
                                            .inventory
                                            .add_in_next_free_slot(transfer_item)
                                    } else {
                                        connection
                                            .player_data
                                            .house_inventory
                                            .add_in_empty_slot(transfer_item, desired_pages)
                                    };

                                    let Some(item_info) = item_info else {
                                        continue;
                                    };

                                    // Have to get this before deleting the item...
                                    let item_id = transfer_item.item_id;

                                    // This is a bit ugly, but we have to do this after that `if` to avoid borrowing the house inventory twice...
                                    let transfer_item = connection
                                        .player_data
                                        .house_inventory
                                        .get_item_mut(storage_id, slot)
                                        .unwrap(); // This unwrap should be fine since the check was performed above.

                                    *transfer_item = Item::default();

                                    // Next, update the client's inventory.
                                    let src_container_type = storage_id;
                                    let dst_container_type = item_info.container;

                                    connection
                                        .send_affected_containers(
                                            src_container_type,
                                            dst_container_type,
                                        )
                                        .await;

                                    // Inform the player via a log message where their item went.
                                    connection
                                        .actor_control_self(ActorControlCategory::LogMessage {
                                            log_message: if to_storeroom {
                                                LogMessageType::FurnitureMovedToStoreroom as u32
                                            } else {
                                                LogMessageType::FurnitureMovedToInventory as u32
                                            },
                                            id: item_id,
                                        })
                                        .await;

                                    // TODO: This probably needs to be networked only if the source furniture was placed in the world, and this is not a transfer from the storeroom to the player's inventory
                                    connection
                                        .broadcast_actor_control(
                                            ActorControlCategory::FurnitureRemovedToInventoryAck {
                                                unk1: slot as u32,
                                                unk2: 0,
                                                unk3: 0,
                                                unk4: 0,
                                            },
                                        )
                                        .await;
                                }
                                ClientTriggerCommand::PlaceFurnitureFromStoreroom {
                                    container_type,
                                    container_index,
                                    ..
                                } => {
                                    let intended_use = lua_player.zone_data.intended_use;
                                    // TODO: Also reject if they're not the owner/shared tenant
                                    if !in_housing_area(intended_use) {
                                        continue;
                                    }

                                    let item_id;
                                    {
                                        let mut game_data = connection.game_data.lock();
                                        let Some(the_item) = connection
                                            .player_data
                                            .house_inventory
                                            .get_item(container_type, container_index)
                                        else {
                                            continue;
                                        };
                                        let Some(the_id) =
                                            game_data.get_furniture_item_id(the_item.item_id)
                                        else {
                                            continue;
                                        };

                                        item_id = the_id;
                                    }

                                    // This acts as a preview, and we don't actually spawn the furniture until the client sends a PlaceFurniture. Therefore, we don't network this.
                                    let indoors =
                                        intended_use == TerritoryIntendedUse::HousingIndoor;
                                    // TODO: implement dyes...
                                    let stain = 0;

                                    if indoors {
                                        connection
                                            .send_ipc_self(ServerZoneIpcSegment::new(
                                                ServerZoneIpcData::InteriorFurniturePlaced {
                                                    storage_id: container_type,
                                                    slot: container_index,
                                                    item_id,
                                                    unk1: 1,
                                                    stain,
                                                    unk2: [0; 3],
                                                    rotation: 0.0,
                                                    position: connection
                                                        .player_data
                                                        .volatile
                                                        .position,
                                                    unk3: [0; 4],
                                                },
                                            ))
                                            .await;
                                    } else {
                                        connection
                                            .send_ipc_self(ServerZoneIpcSegment::new(
                                                ServerZoneIpcData::ExteriorFurniturePlaced {
                                                    plot_index: 0xFF, // During preview mode, the plot index and slot are 0xFF.
                                                    slot: 0xFF,
                                                    unk1: [0; 2],
                                                    item_id,
                                                    unk2: 0,
                                                    stain,
                                                    unk3: [0; 3],
                                                    rotation: 0.0,
                                                    position: connection
                                                        .player_data
                                                        .volatile
                                                        .position,
                                                    unk4: 0,
                                                },
                                            ))
                                            .await;
                                    }
                                }
                                ClientTriggerCommand::RequestPlayerName => {
                                    if let ClientTriggerTarget::ContentId(content_id) =
                                        trigger.target
                                    {
                                        connection
                                            .send_ipc_self(ServerZoneIpcSegment::new(
                                                ServerZoneIpcData::PlayerName {
                                                    content_id,
                                                    name: "It's a mystery".to_string(), // Dummied out name for now
                                                },
                                            ))
                                            .await;
                                    }
                                }
                                ClientTriggerCommand::TeleportOfferReply { decline_teleport } => {
                                    if !decline_teleport
                                        && let Some(offered_teleport) = &connection.offered_teleport
                                    {
                                        connection
                                            .warp_aetheryte(
                                                offered_teleport.aetheryte_id as u32,
                                                false,
                                                true,
                                                WarpType::Teleport,
                                            )
                                            .await;
                                    } else {
                                        connection.offered_teleport = None;
                                    }
                                }
                                ClientTriggerCommand::OpenSharedFATEWindow { page } => {
                                    let start_index = TAB_SHARED_FATE_COUNT * page as usize;
                                    let fate_counts = &connection.player_data.quest.shared_fates.0
                                        [start_index..start_index + 6];
                                    let mut fate_count_maximum = [0u8; 6];
                                    {
                                        for (j, i) in (start_index..start_index + 6).enumerate() {
                                            let mut game_data = connection.game_data.lock();
                                            fate_count_maximum[j] = game_data
                                                .get_fate_progress_maximum_count(i as u32)
                                                .unwrap_or_default();
                                        }
                                    }

                                    let ipc = ServerZoneIpcSegment::new(
                                        ServerZoneIpcData::SharedFATEInformation {
                                            page: page as u8,
                                            fate_count: fate_counts.try_into().unwrap(),
                                            fate_count_maximum,
                                        },
                                    );
                                    connection.send_ipc_self(ipc).await;
                                }
                                ClientTriggerCommand::FateLevelSync { fate_id, unk1, .. } => {
                                    // unk1 is 1 to sync (all of Bob's retail captures), 0 to unsync.
                                    connection.set_fate_level_sync(fate_id, unk1 != 0).await;
                                }
                                ClientTriggerCommand::EquipGlasses { slot, id } => {
                                    connection.player_data.equipped_glasses_ids[slot as usize] =
                                        id as u16;

                                    connection
                                        .handle
                                        .send(ToServer::EquipGlasses(
                                            connection.player_data.character.actor_id,
                                            slot,
                                            id,
                                        ))
                                        .await;
                                }
                                ClientTriggerCommand::RegisterDutySupport {
                                    dawn_content_id,
                                    unk1,
                                    unk2,
                                    unk3,
                                    unk4,
                                } => {
                                    let content_finder_condition_id;
                                    {
                                        let mut game_data = connection.game_data.lock();
                                        content_finder_condition_id = game_data
                                            .get_dawn_content_cfc_id(dawn_content_id)
                                            .unwrap();
                                    }

                                    connection
                                        .register_for_content(
                                            [content_finder_condition_id as u16, 0, 0, 0, 0],
                                            DutyFinderSetting::default(),
                                        )
                                        .await;
                                    // The members chosen, one byte each (Bob's capture: 0x00252624, 36, 38, 37.)
                                    connection.duty_support = Some(kawari_world::chosen_members([
                                        unk1, unk2, unk3, unk4,
                                    ]));
                                }
                                ClientTriggerCommand::StartOrStopPerforming { perform_id } => {
                                    if perform_id == 0 {
                                        connection
                                            .set_character_mode(CharacterMode::Normal, 0, true)
                                            .await;
                                    } else {
                                        connection
                                            .set_character_mode(
                                                CharacterMode::Performance,
                                                perform_id as u8,
                                                true,
                                            )
                                            .await;
                                    }
                                }
                                ClientTriggerCommand::ZoneDiceRoll { num_sides } => {
                                    connection.send_dice_roll(num_sides as u16).await;
                                }
                                // A teleport the character can't pay for is refused straight away (the cost is
                                // taken when it lands, see `pay_for_teleport`.)
                                ClientTriggerCommand::TeleportQuery { aetheryte_id, .. }
                                    if connection.teleport_cost(aetheryte_id)
                                        > connection
                                            .player_data
                                            .inventory
                                            .currency
                                            .gil
                                            .quantity
                                            .max(0)
                                            as u32 =>
                                {
                                    connection
                                        .actor_control_self(ActorControlCategory::TeleportStart {
                                            insufficient_gil: TELEPORT_INSUFFICIENT_GIL,
                                            aetheryte_id,
                                        })
                                        .await;
                                }
                                _ => {
                                    // inform the server of our trigger, it will handle sending it to other clients
                                    connection
                                        .handle
                                        .send(ToServer::ClientTrigger(
                                            connection.id,
                                            connection.player_data.character.actor_id,
                                            trigger.clone(),
                                        ))
                                        .await;
                                }
                            }
                        }
                        ClientZoneIpcData::UnkSocialEvent { .. } => {
                            connection
                                .send_ipc_self(ServerZoneIpcSegment::new(
                                    ServerZoneIpcData::UnkSocialResponse {
                                        unk: Default::default(),
                                    },
                                ))
                                .await;
                        }
                        ClientZoneIpcData::SocialListRequest(request) => {
                            if request.request_type == SocialListRequestType::Friends {
                                // We have to refresh manually here as the friend list doesn't have a convenient search & result opcode pair like searching does.
                                connection.refresh_friend_list().await;
                            }
                            let entries = if request.request_type == SocialListRequestType::Party {
                                Some(connection.party_member_entries())
                            } else {
                                None
                            };

                            connection
                                .send_social_list(
                                    request.request_type,
                                    request.sequence,
                                    entries,
                                    None,
                                )
                                .await;
                        }
                        ClientZoneIpcData::UpdatePositionHandler {
                            position,
                            rotation,
                            anim_type,
                            anim_state,
                            jump_state,
                        } => {
                            if connection.spawned_in {
                                connection.player_data.volatile.rotation = *rotation as f64;
                                connection.player_data.volatile.position = *position;

                                let party_id = if connection.party_id != 0 {
                                    Some(connection.party_id)
                                } else {
                                    None
                                };

                                connection
                                    .handle
                                    .send(ToServer::ActorMoved(
                                        connection.player_data.character.actor_id,
                                        *position,
                                        *rotation,
                                        *anim_type,
                                        *anim_state,
                                        *jump_state,
                                        party_id,
                                    ))
                                    .await;
                            }
                        }
                        ClientZoneIpcData::LogOut { .. } => {
                            connection.gracefully_logged_out = true;
                            connection.begin_log_out().await;
                        }
                        ClientZoneIpcData::Disconnected { .. } => {
                            tracing::info!("Client disconnected!");

                            // We no longer send ToServer::Disconnected here because the end of the function already does it unconditionally
                            return false;
                        }
                        ClientZoneIpcData::SendChatMessage(chat_message) => {
                            // Process debug commands
                            if chat_message
                                .message
                                .to_string()
                                .starts_with(DEBUG_COMMAND_TRIGGER)
                            {
                                let msg = chat_message.message.to_string();
                                let parts: Vec<&str> = msg.split(' ').collect();
                                let command_name = &parts[0][1..];

                                {
                                    let lua = connection.lua.lock();
                                    let state = lua.0.app_data_ref::<KawariLuaState>().unwrap();

                                    // If a Lua command exists, try using that first
                                    if let Some(command_script) =
                                        state.command_scripts.get(command_name)
                                    {
                                        let file_name = get_config()
                                            .filesystem
                                            .locate_script_file(command_script);

                                        let mut run_script = || -> mlua::Result<()> {
                                            lua.0.scope(|scope| {
                                                    let connection_data = scope
                                                    .create_userdata_ref_mut(lua_player)?;

                                                    /* TODO: Instead of panicking we ought to send a message to the player
                                                     * and the console log, and abandon execution. */
                                                    lua.0.load(
                                                        std::fs::read(&file_name).unwrap_or_else(|_| panic!("Failed to load script file {}!", file_name)),
                                                    )
                                                    .set_name("@".to_string() + &file_name)
                                                    .exec()?;

                                                    let required_rank = lua.0.globals().get("required_rank");
                                                    if let Err(error) = required_rank {
                                                        tracing::info!("Script is missing required_rank! Unable to run command, sending error to user. Additional information: {}", error);
                                                        let func: Function =
                                                        lua.0.globals().get("onCommandRequiredRankMissingError")?;
                                                        func.call::<()>((error.to_string(), connection_data))?;
                                                        return Ok(());
                                                    }

                                                    /* Reset state for future commands. Without this it'll stay set to the last value
                                                     * and allow other commands that omit required_rank to run, which is undesirable. */
                                                    lua.0.globals().set("required_rank", mlua::Value::Nil)?;

                                                    if connection.player_data.character.gm_rank as u8 >= required_rank? {
                                                        let mut func_args = Vec::new();
                                                        if parts.len() > 1 {
                                                            func_args = (parts[1..]).to_vec();
                                                            tracing::info!("Args passed to Lua command {}: {:?}", command_name, func_args);
                                                        } else {
                                                            tracing::info!("No additional args passed to Lua command {}.", command_name);
                                                        }
                                                        let func: Function =
                                                        lua.0.globals().get("onCommand")?;
                                                        func.call::<()>((connection_data, func_args, String::default()))?;

                                                        /* `command_sender` is an optional variable scripts can define to identify themselves in print messages.
                                                         * It's okay if this global isn't set. We also don't care what its value is, just that it exists.
                                                         * This is reset -after- running the command intentionally. Resetting beforehand will never display the command's identifier.
                                                         */
                                                        let command_sender: Result<mlua::prelude::LuaValue, mlua::prelude::LuaError> = lua.0.globals().get("command_sender");
                                                        if command_sender.is_ok() {
                                                            lua.0.globals().set("command_sender", mlua::Value::Nil)?;
                                                        }
                                                        Ok(())
                                                    } else {
                                                        tracing::info!("User with account_id {} tried to invoke GM command {} with insufficient privileges!",
                                                                       connection.player_data.character.service_account_id, command_name);
                                                        let func: Function =
                                                        lua.0.globals().get("onCommandRequiredRankInsufficientError")?;
                                                        func.call::<()>(connection_data)?;
                                                        Ok(())
                                                    }
                                                })
                                        };

                                        if let Err(err) = run_script() {
                                            tracing::warn!("Lua error in {file_name}: {:?}", err);
                                        }

                                        continue; // Don't send the message off anywhere
                                    }
                                }

                                // Fallback to Rust implemented commands in ZoneConnection, but if this fails then
                                if connection
                                    .process_debug_commands(&chat_message.message, events)
                                    .await
                                {
                                    continue; // Don't send the message off anywhere
                                }
                            }

                            // Send the message to the global server to be processed further
                            let info = MessageInfo {
                                sender_actor_id: connection.player_data.character.actor_id,
                                sender_account_id: connection
                                    .player_data
                                    .character
                                    .service_account_id
                                    as u64,
                                sender_world_id: connection.config.world_id,
                                sender_position: connection.player_data.volatile.position,
                                sender_name: connection.player_data.character.name.clone(),
                                channel: chat_message.channel,
                                message: chat_message.message.clone(),
                            };

                            connection
                                .handle
                                .send(ToServer::Message(
                                    connection.id,
                                    connection.player_data.character.actor_id,
                                    info,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::GMCommand {
                            command,
                            arg0,
                            arg1,
                            arg2,
                            arg3,
                            ..
                        } => {
                            connection
                                .run_gm_command(
                                    *command,
                                    *arg0,
                                    *arg1,
                                    *arg2,
                                    *arg3,
                                    String::default(),
                                    lua_player,
                                )
                                .await;
                        }
                        ClientZoneIpcData::GMCommandName {
                            command,
                            arg0,
                            arg1,
                            arg2,
                            arg3,
                            name,
                            ..
                        } => {
                            connection
                                .run_gm_command(
                                    *command,
                                    *arg0,
                                    *arg1,
                                    *arg2,
                                    *arg3,
                                    name.clone(),
                                    lua_player,
                                )
                                .await;
                        }
                        ClientZoneIpcData::GMCommandName2 { data, .. } => {
                            tracing::info!("GM Command Test: {data}");
                        }
                        ClientZoneIpcData::ZoneJump {
                            exit_box, position, ..
                        } => {
                            tracing::info!(
                                "Character entered {exit_box} with a position of {position:#?}!"
                            );

                            connection
                                .handle
                                .send(ToServer::EnterZoneJump(
                                    connection.id,
                                    connection.player_data.character.actor_id,
                                    *exit_box,
                                    WarpType::Normal,
                                    0,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::ActionRequest(request) => {
                            connection
                                .handle
                                .send(ToServer::ActionRequest(
                                    connection.id,
                                    connection.player_data.character.actor_id,
                                    request.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::ActionRequestGroundTargeted {
                            action_id,
                            unk1,
                            action_type,
                            sequence,
                            rotation,
                            ..
                        } => {
                            // Handled like an action on themselves, so the client gets its result (ground placement,
                            // e.g. Asylum's area, isn't done yet.)
                            let request = kawari::ipc::zone::ActionRequest {
                                action_id: *action_id,
                                unk1: *unk1,
                                action_type: *action_type,
                                sequence: *sequence,
                                rotation1: *rotation,
                                rotation2: *rotation,
                                target: ObjectTypeId {
                                    object_id: connection.player_data.character.actor_id,
                                    object_type: ObjectTypeKind::None,
                                },
                                ..Default::default()
                            };
                            connection
                                .handle
                                .send(ToServer::ActionRequest(
                                    connection.id,
                                    connection.player_data.character.actor_id,
                                    request,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::PingSync { timestamp, .. } => {
                            // this is *usually* sent in response, but not always
                            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::PingSyncReply {
                                timestamp: *timestamp,      // copied from here
                                transmission_interval: 333, // always this for some reason
                            });
                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::ItemOperation(action)
                            if matches!(
                                action.operation_type,
                                ItemOperationKind::RetainerEntrust
                                    | ItemOperationKind::RetainerWithdraw
                                    | ItemOperationKind::RetainerGil
                            ) =>
                        {
                            connection.retainer_item_operation(action).await;
                        }
                        ClientZoneIpcData::ItemOperation(action)
                            if matches!(
                                action.operation_type,
                                ItemOperationKind::SaddlebagStore
                                    | ItemOperationKind::SaddlebagTake
                            ) =>
                        {
                            connection.saddlebag_item_operation(action).await;
                        }
                        ClientZoneIpcData::ItemOperation(action) => {
                            tracing::info!("Client is modifying inventory! {action:#?}");
                            connection
                                .send_inventory_ack(
                                    action.context_id,
                                    INVENTORY_ACTION_ACK_GENERAL,
                                    0,
                                )
                                .await;

                            connection.player_data.inventory.process_action(action);
                            if matches!(
                                action.operation_type,
                                ItemOperationKind::RetainerMarketList
                                    | ItemOperationKind::RetainerMarketListCrystal
                            ) {
                                connection.mark_market_changed();
                            }
                            if retainer::is_retainer_container(action.src_storage_id)
                                || retainer::is_retainer_container(action.dst_storage_id)
                            {
                                connection.save_called_retainer();
                            }

                            if action.operation_type == ItemOperationKind::Discard {
                                tracing::info!("Client is discarding from their inventory!");

                                let ipc = ServerZoneIpcSegment::new(
                                    ServerZoneIpcData::InventoryTransaction {
                                        sequence: connection.player_data.item_sequence,
                                        operation_type: action.operation_type,
                                        src_actor_id: connection.player_data.character.actor_id,
                                        src_storage_id: action.src_storage_id,
                                        src_container_index: action.src_container_index,
                                        src_stack: action.src_stack,
                                        src_item_id: action.src_item_id,
                                        dummy_container: ContainerType::DiscardingItemSentinel,
                                        dst_storage_id: ContainerType::DiscardingItemSentinel,
                                        dst_container_index: u16::MAX,
                                        dst_actor_id: ObjectId::default(),
                                        dst_stack: 0,
                                        dst_item_id: 0,
                                    },
                                );
                                connection.send_ipc_self(ipc).await;
                                connection
                                    .send_inventory_transaction_finish(
                                        ItemOperationKind::UnkDiscard,
                                        0x200,
                                    )
                                    .await;
                            }

                            connection.player_data.item_sequence += 1;

                            // This is annoying, basically if we change weapons the client expects the *server* to also equip the relevant job crystal if available.
                            // The client DOES not send us an ItemOperation for this, but in turn we don't have to inform them about the update.
                            if (action.src_storage_id == ContainerType::Equipped
                                && action.src_container_index == EquipSlot::MainHand as u16)
                                || (action.dst_storage_id == ContainerType::Equipped
                                    && action.dst_container_index == EquipSlot::MainHand as u16)
                            {
                                tracing::info!("Changing class based on weapon...");

                                // We need to update our current class based on the weapon...
                                connection.change_class_based_on_weapon().await;

                                tracing::info!(
                                    "- Result: {}",
                                    connection.player_data.classjob.current_class
                                );

                                let id;
                                {
                                    let mut game_data = connection.game_data.lock();
                                    id = game_data.get_soul_crystal_item_id(
                                        connection.player_data.classjob.current_class as u16,
                                    );
                                }
                                if let Some(id) = id
                                    && connection.player_data.inventory.has_soul_crystal(id)
                                {
                                    tracing::info!(
                                        "*We* are equipping soul crystal {id} based on client behavior..."
                                    );
                                    connection.player_data.inventory.equip_soul_crystal(id);

                                    // Then re-check the soul crystal...
                                    connection.change_class_based_on_soul_crystal().await;
                                } else {
                                    tracing::info!(
                                        "*We* are unequipping the soul crystal based on client behavior..."
                                    );
                                    connection
                                        .player_data
                                        .inventory
                                        .unequip_equipment(EquipSlot::SoulCrystal as u16);
                                }
                            }

                            // If the soul crystal is changed, ensure we update accordingly.
                            if (action.src_storage_id == ContainerType::Equipped
                                && action.src_container_index == EquipSlot::SoulCrystal as u16)
                                || (action.dst_storage_id == ContainerType::Equipped
                                    && action.dst_container_index == EquipSlot::SoulCrystal as u16)
                            {
                                let soul_crystal =
                                    connection.player_data.inventory.equipped.soul_crystal;
                                if soul_crystal.quantity > 0 {
                                    tracing::info!(
                                        "We are re-checking class based on the soul crystal..."
                                    );
                                    connection.change_class_based_on_soul_crystal().await;
                                } else {
                                    tracing::info!(
                                        "We are re-checking class based on the weapon..."
                                    );
                                    connection.change_class_based_on_weapon().await;
                                }
                            }

                            // This is also something done client-side.
                            connection.remove_incompatible_armor(action).await;

                            // If the client modified their equipped items, we have to process that
                            if action.src_storage_id == ContainerType::Equipped
                                || action.dst_storage_id == ContainerType::Equipped
                            {
                                connection.inform_equip().await;
                                connection.send_stats().await; // Because stats changed based on equipped items
                            }
                        }
                        ClientZoneIpcData::StartTalkEvent {
                            actor_id,
                            handler_id,
                        } => {
                            if connection
                                .start_event(
                                    *actor_id,
                                    *handler_id,
                                    EventType::Talk,
                                    0,
                                    events,
                                    lua_player,
                                )
                                .await
                            {
                                // Something to pick up (a GimmickAccessor of the Interactable kind: a charge, a key, an
                                // item lying about) asks nothing on retail: the event starts, its interaction runs and
                                // the event finishes, with no scene (Bob's Copperbell Mines captures of 2026-10-25:
                                // all eight kinds there). Kawari asked yes or no first.
                                let pickup = handler_id.handler_type()
                                    == HandlerType::GimmickAccessor
                                    && matches!(
                                        connection
                                            .game_data
                                            .lock()
                                            .lookup_gimmick_accessor(handler_id.event_id()),
                                        Some((GimmickAccessorType::Interactable, ..))
                                    );
                                if pickup {
                                    connection
                                        .handle
                                        .send(ToServer::GimmickAccessor(
                                            handler_id.event_id(),
                                            connection.player_data.character.actor_id,
                                            actor_id.object_id,
                                            Vec::new(),
                                        ))
                                        .await;
                                } else if let Some(event) = events.last_mut() {
                                    // begin talk function if it exists (handlers that need the connection first)
                                    if !event
                                        .0
                                        .on_talk_connection(
                                            &event.1, *actor_id, connection, lua_player,
                                        )
                                        .await
                                    {
                                        event.0.on_talk(&event.1, *actor_id, lua_player).await;
                                    }
                                }
                            }
                        }
                        ClientZoneIpcData::EventReturnHandler2(handler) => {
                            tracing::info!(message = "Event returned", handler_id = %handler.handler_id, error_code = handler.error_code, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);
                            lua_player.scene_error_code = handler.error_code;

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_return(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to return from!");
                            }
                        }
                        ClientZoneIpcData::EventReturnHandler8(handler) => {
                            tracing::info!(message = "Event returned", handler_id = %handler.handler_id, error_code = handler.error_code, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);
                            lua_player.scene_error_code = handler.error_code;

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_return(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to return from!");
                            }
                        }
                        ClientZoneIpcData::EventReturnHandler16(handler) => {
                            tracing::info!(message = "Event returned", handler_id = %handler.handler_id, error_code = handler.error_code, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);
                            lua_player.scene_error_code = handler.error_code;

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_return(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to return from!");
                            }
                        }
                        ClientZoneIpcData::EventReturnHandler128(handler) => {
                            tracing::info!(message = "Event returned", handler_id = %handler.handler_id, error_code = handler.error_code, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);
                            lua_player.scene_error_code = handler.error_code;

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_return(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to return from!");
                            }
                        }
                        ClientZoneIpcData::EventYieldHandler2(handler) => {
                            tracing::info!(message = "Event yielded", handler_id = %handler.handler_id, yield_id = handler.yield_id, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_yield(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        handler.yield_id,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to yield from!");
                            }
                        }
                        ClientZoneIpcData::EventYieldHandler4(handler) => {
                            tracing::info!(message = "Event yielded", handler_id = %handler.handler_id, yield_id = handler.yield_id, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_yield(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        handler.yield_id,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to yield from!");
                            }
                        }
                        ClientZoneIpcData::EventYieldHandler16(handler) => {
                            tracing::info!(message = "Event yielded", handler_id = %handler.handler_id, yield_id = handler.yield_id, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_yield(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        handler.yield_id,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to yield from!");
                            }
                        }
                        ClientZoneIpcData::EventYieldHandler32(handler) => {
                            tracing::info!(message = "Event yielded", handler_id = %handler.handler_id, yield_id = handler.yield_id, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_yield(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        handler.yield_id,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to yield from!");
                            }
                        }
                        ClientZoneIpcData::EventYieldHandler128(handler) => {
                            tracing::info!(message = "Event yielded", handler_id = %handler.handler_id, yield_id = handler.yield_id, scene = handler.scene, params = ?&handler.params[..handler.num_results as usize]);

                            if let Some(event) = events.last() {
                                event
                                    .0
                                    .on_yield(
                                        &event.1,
                                        connection,
                                        handler.scene,
                                        handler.yield_id,
                                        &handler.params[..handler.num_results as usize],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("There's no current event to yield from!");
                            }
                        }
                        ClientZoneIpcData::Config(config) => {
                            // Update our own state so it's committed on log out
                            connection.player_data.volatile.display_flags = config.display_flag;
                            connection
                                .handle
                                .send(ToServer::Config(
                                    connection.id,
                                    connection.player_data.character.actor_id,
                                    config.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::StandardControlsPivot { .. } => {
                            /* No-op because we already seem to handle this, other nearby clients can see the sending player
                             * pivoting anyway. */
                        }
                        ClientZoneIpcData::UnkCall2 { .. } => {
                            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UnkResponse2 {
                                unk1: 1,
                            });
                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::QueueDuties(queue_duties) => {
                            connection
                                .register_for_content(
                                    queue_duties.content_ids,
                                    queue_duties.settings,
                                )
                                .await;
                        }
                        ClientZoneIpcData::QueueRoulette {
                            roulette_id, unk1, ..
                        } => {
                            let Some(roulette) = Roulette::from_repr(*roulette_id) else {
                                tracing::warn!("Unknown roulette ID: {roulette_id}");
                                continue;
                            };
                            // The settings: assumed to sit where QueueDuties has them (its byte 8; no QueueRoulette
                            // is recorded yet), and only those a roulette can use are taken
                            // (`duty_finder::roulette_settings`.)
                            let mut raw = [0u8; 8];
                            raw.copy_from_slice(&unk1[7..15]);
                            let settings = kawari_world::duty_finder::roulette_settings(
                                DutyFinderSetting::from_bits_retain(u64::from_le_bytes(raw)),
                            );
                            tracing::info!(
                                "QueueRoulette {roulette_id}: bytes after the ID {unk1:02x?}, settings taken {settings:?}"
                            );

                            let duty_id;
                            {
                                let mut game_data = connection.game_data.lock();
                                duty_id = connection.pick_roulette_duty(
                                    &mut game_data,
                                    roulette,
                                    settings.contains(DutyFinderSetting::LIMITED_LEVELING_ROULETTE),
                                );
                            }
                            let Some(duty_id) = duty_id else {
                                tracing::info!(
                                    "Roulette {roulette_id}: no duty in it is unlocked for this character at their level"
                                );
                                continue;
                            };

                            connection
                                .register_for_content([duty_id, 0, 0, 0, 0], settings)
                                .await;
                            connection.queued_roulette = Some(*roulette_id);
                        }
                        ClientZoneIpcData::ContentFinderAction { action, .. } => {
                            if *action == ContentFinderUserAction::Accepted {
                                // commencing: state 4, 1, the duty (ContentFinderCondition) as a u32 at byte 8,
                                // then how many are ready and the party's size, as retail (Bob's Chronofoil captures;
                                // Kawari had a 4 where the duty goes.)
                                {
                                    let cfc = connection.queued_content.unwrap_or_default() as u32;
                                    let mut unk1 = [
                                        4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
                                        0, 0, 0, 0,
                                    ];
                                    unk1[8..12].copy_from_slice(&cfc.to_le_bytes());
                                    let ipc = ServerZoneIpcSegment::new(
                                        ServerZoneIpcData::ContentFinderCommencing { unk1 },
                                    );
                                    connection.send_ipc_self(ipc).await;
                                }

                                connection
                                    .join_content(
                                        connection.queued_content.unwrap(),
                                        connection.duty_settings.unwrap(),
                                    )
                                    .await;
                            }

                            // If we don't send this, the content finder gets stuck.
                            let ipc =
                                ServerZoneIpcSegment::new(ServerZoneIpcData::UnkContentFinder {
                                    unk: [0; 16],
                                });
                            connection.send_ipc_self(ipc).await;

                            connection.queued_content = None;
                        }
                        ClientZoneIpcData::SetAdventurerPlatePortrait { portrait } => {
                            // Saved with the character and sent back whenever the client asks for it.
                            connection.player_data.volatile.portrait.data = portrait.to_vec();
                        }
                        ClientZoneIpcData::UpdateAdventurerPlate {
                            section,
                            chara_card,
                            ..
                        } => {
                            // Saved straight away: the client asks for the plate again right after saving it, and
                            // other players' requests read it from the database. As on retail (2026-10-20 capture),
                            // unk14 is the client's leftover memory and is stored as 0, and the save time goes back
                            // to the client as the plate's timestamp.
                            tracing::info!("Adventurer plate saved (section {section})");
                            let mut chara_card = chara_card.clone();
                            chara_card.unk14 = 0;
                            connection.player_data.volatile.adventurer_plate.data =
                                chara_card.to_bytes();
                            connection.player_data.volatile.adventurer_plate_saved =
                                kawari::common::timestamp_secs() as i64;
                            let mut database = connection.database.lock();
                            database.commit_volatile(&connection.player_data);
                        }
                        ClientZoneIpcData::EquipGearset {
                            gearset_index,
                            containers,
                            indices,
                            ..
                        } => {
                            connection
                                .equip_gearset(*gearset_index, containers, indices)
                                .await;
                        }
                        // A gearset with a glamour plate linked: equipped, then the plate applied (Bob's capture of
                        // 2026-10-24.)
                        ClientZoneIpcData::EquipGearsetGlamour {
                            gearset_index,
                            containers,
                            indices,
                            glamour_plate,
                            ..
                        } => {
                            connection
                                .equip_gearset(*gearset_index, containers, indices)
                                .await;
                            if *glamour_plate > 0 {
                                connection
                                    .apply_glamour_plate(*glamour_plate as usize - 1)
                                    .await;
                            }
                        }
                        ClientZoneIpcData::EquipGearset2 { .. } => {
                            tracing::warn!("Bigger gearsets not supported yet!");
                        }
                        ClientZoneIpcData::StartWalkInEvent {
                            event_arg,
                            handler_id,
                            ..
                        } => {
                            // Yes, an ActorControl is sent here, not an ActorControlSelf!
                            connection
                                .actor_control(
                                    connection.player_data.character.actor_id,
                                    ActorControlCategory::ToggleWeapon {
                                        shown: false,
                                        unk_flag: 1,
                                        unk2: 1,
                                    },
                                )
                                .await;

                            let actor_id = ObjectTypeId {
                                object_id: connection.player_data.character.actor_id,
                                object_type: ObjectTypeKind::None,
                            };
                            connection
                                .start_event(
                                    actor_id,
                                    *handler_id,
                                    EventType::WithinRange,
                                    *event_arg,
                                    events,
                                    lua_player,
                                )
                                .await;

                            // begin walk-in trigger function if it exists
                            if let Some(event) = events.last_mut() {
                                event
                                    .0
                                    .on_enter_trigger(&event.1, lua_player, *event_arg)
                                    .await;
                            }
                        }
                        ClientZoneIpcData::WalkOutsideEvent {
                            event_arg,
                            handler_id,
                            ..
                        } => {
                            // TODO: allow Lua scripts to handle these differently?

                            // Yes, an ActorControl is sent here, not an ActorControlSelf!
                            connection
                                .actor_control(
                                    connection.player_data.character.actor_id,
                                    ActorControlCategory::ToggleWeapon {
                                        shown: false,
                                        unk_flag: 1,
                                        unk2: 1,
                                    },
                                )
                                .await;

                            let actor_id = ObjectTypeId {
                                object_id: connection.player_data.character.actor_id,
                                object_type: ObjectTypeKind::None,
                            };
                            connection
                                .start_event(
                                    actor_id,
                                    *handler_id,
                                    EventType::OutsideRange,
                                    *event_arg,
                                    events,
                                    lua_player,
                                )
                                .await;

                            // begin walk-in trigger function if it exists
                            if let Some(event) = events.last_mut() {
                                event
                                    .0
                                    .on_enter_trigger(&event.1, lua_player, *event_arg)
                                    .await;
                            }
                        }
                        ClientZoneIpcData::NewDiscovery { layout_id, pos } => {
                            tracing::info!(
                                "Client discovered a new location on {:?} at {:?}!",
                                layout_id,
                                pos
                            );

                            connection
                                .handle
                                .send(ToServer::NewLocationDiscovered(
                                    connection.id,
                                    *layout_id,
                                    *pos,
                                    connection.player_data.volatile.zone_id as u16,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::RequestBlacklist(request) => {
                            // TODO: Actually implement this beyond simply sending a blank list
                            // NOTE: Failing to respond to this request means SpawnPlayer will not work and other players will be invisible, have their chat ignored and possibly other issues by the client! Beware!
                            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::Blacklist(
                                Blacklist {
                                    data: vec![
                                        BlacklistedCharacter::default();
                                        Blacklist::NUM_ENTRIES
                                    ],
                                    sequence: request.sequence,
                                },
                            ));
                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::RequestFellowships { .. } => {
                            // Report back that they're not in a fellowship
                            // TODO: Implement this for real!
                            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::Fellowships {
                                unk1: [0; 808],
                            });
                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::RequestCrossworldLinkshells { .. } => {
                            connection.send_crossworld_linkshells(true).await;
                        }
                        ClientZoneIpcData::SearchFellowships { .. } => {
                            // TODO: Actually implement Fellowship searches, for now none of the player's search criteria are used.
                            let fellowships = vec![
                                FellowshipSearchInfo {
                                    community_id: 1,
                                    master_content_id: 1,
                                    recruiter_content_id: 1,
                                    recruitment_deadline: 0xFFFF_FFFF,
                                    languages: SocialListUILanguages::JAPANESE
                                        | SocialListUILanguages::ENGLISH
                                        | SocialListUILanguages::GERMAN
                                        | SocialListUILanguages::FRENCH,
                                    primary_language: ClientLanguage::English,
                                    recruiter_world_id: connection.config.world_id,
                                    master_world_id: connection.config.world_id,
                                    current_member_count: 999,
                                    target_member_count: 1000,
                                    activity1: FellowshipActivityTag::Chatting,
                                    unk1: [0; 4],
                                    activity2: FellowshipActivityTag::DomanMahjong,
                                    activity3: FellowshipActivityTag::Glamours,
                                    fellowship_name: "Kawari Fellowship Test".to_string(),
                                    recruiter_name: "Kawari Player".to_string(),
                                    master_name: "Kawari Player".to_string(),
                                    fellowship_description: "The Fellowship's description.".into(),
                                },
                                FellowshipSearchInfo::default(),
                                FellowshipSearchInfo::default(),
                                FellowshipSearchInfo::default(),
                            ];

                            let ipc = ServerZoneIpcSegment::new(
                                ServerZoneIpcData::SearchFellowshipsResults {
                                    unk1: 0xFFFF_FFFF,
                                    sequence_end: 0xFFFF_FFFF,
                                    sequence_current: 0,
                                    fellowships,
                                },
                            );

                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::StartCountdown {
                            starter_actor_id,
                            duration,
                            starter_name,
                        } => {
                            connection
                                .handle
                                .send(ToServer::StartCountdown(
                                    connection.party_id,
                                    connection.player_data.character.actor_id,
                                    connection.player_data.character.service_account_id as u64,
                                    connection.player_data.character.content_id as u64,
                                    starter_name.clone(),
                                    *starter_actor_id,
                                    *duration,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::RequestPlaytime { .. } => {
                            connection.send_playtime().await;
                        }
                        ClientZoneIpcData::SetFreeCompanyGreeting { .. } => {
                            tracing::info!("Setting the free company greeting is unimplemented");
                        }
                        ClientZoneIpcData::SetClientLanguage { language } => {
                            connection.player_data.volatile.client_language = *language;

                            // We need to commit this to the database again since this is read everywhere.
                            {
                                let mut database = connection.database.lock();
                                database.commit_volatile(&connection.player_data);
                            }
                        }
                        ClientZoneIpcData::RequestCharaInfoFromContentIds { .. } => {
                            tracing::info!(
                                "Requesting character info from content ids is unimplemented"
                            );
                        }
                        ClientZoneIpcData::InviteCharacter {
                            content_id,
                            invite_type,
                            character_name,
                            ..
                        } => {
                            tracing::info!(
                                "Client invited a character! {:#?} {:#?} {:#?} {:#?}",
                                content_id,
                                invite_type,
                                character_name,
                                data.data
                            );
                            connection
                                .send_social_invite(
                                    *content_id,
                                    *invite_type,
                                    character_name.clone(),
                                )
                                .await;
                        }
                        ClientZoneIpcData::InviteReply {
                            inviter_content_id,
                            invite_type,
                            response,
                            ..
                        }
                        | ClientZoneIpcData::InviteReply2 {
                            inviter_content_id,
                            response,
                            invite_type,
                            ..
                        } => {
                            tracing::info!(
                                "Client replied to invite: {:#?} {:#?} {:#?}",
                                inviter_content_id,
                                invite_type,
                                response
                            );
                            connection
                                .invite_reply(*inviter_content_id, *invite_type, *response)
                                .await;
                        }
                        // In no party the server keeps (`party_id` 0): a party of bots, or none. The party code
                        // panics on party 0, so these go to the bots' instead (`party_bot.rs`.)
                        ClientZoneIpcData::PartyDisband { .. }
                        | ClientZoneIpcData::PartyLeave { .. }
                            if connection.party_id == 0 =>
                        {
                            connection
                                .handle
                                .send(ToServer::BotPartyAction(
                                    connection.player_data.character.actor_id,
                                    BotPartyAction::Leave,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::PartyMemberKick {
                            content_id,
                            character_name,
                            ..
                        } if connection.party_id == 0 => {
                            connection
                                .handle
                                .send(ToServer::BotPartyAction(
                                    connection.player_data.character.actor_id,
                                    BotPartyAction::Kick {
                                        content_id: *content_id,
                                        name: character_name.clone(),
                                    },
                                ))
                                .await;
                        }
                        ClientZoneIpcData::PartyChangeLeader { content_id, .. }
                            if connection.party_id == 0 =>
                        {
                            connection
                                .handle
                                .send(ToServer::BotPartyAction(
                                    connection.player_data.character.actor_id,
                                    BotPartyAction::Promote {
                                        content_id: *content_id,
                                    },
                                ))
                                .await;
                        }
                        ClientZoneIpcData::PartyDisband { .. } => {
                            tracing::info!("Client is disbanding their party!");
                            connection
                                .handle
                                .send(ToServer::PartyDisband(
                                    connection.party_id,
                                    connection.player_data.character.service_account_id as u64,
                                    connection.player_data.character.content_id as u64,
                                    connection.player_data.character.name.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::PartyMemberKick {
                            content_id,
                            character_name,
                            ..
                        } => {
                            tracing::info!(
                                "Player is kicking another player from their party! {} {}",
                                content_id,
                                character_name
                            );
                            connection
                                .handle
                                .send(ToServer::PartyMemberKick(
                                    connection.party_id,
                                    connection.player_data.character.service_account_id as u64,
                                    connection.player_data.character.content_id as u64,
                                    connection.player_data.character.name.clone(),
                                    *content_id,
                                    character_name.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::PartyChangeLeader {
                            content_id,
                            character_name,
                            ..
                        } => {
                            tracing::info!(
                                "Player is promoting another player in their party to leader! {} {}",
                                content_id,
                                character_name
                            );
                            connection
                                .handle
                                .send(ToServer::PartyChangeLeader(
                                    connection.party_id,
                                    connection.player_data.character.service_account_id as u64,
                                    connection.player_data.character.content_id as u64,
                                    connection.player_data.character.name.clone(),
                                    *content_id,
                                    character_name.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::PartyLeave { .. } => {
                            tracing::info!("Client is leaving their party!");
                            connection
                                .handle
                                .send(ToServer::PartyMemberLeft(
                                    connection.party_id,
                                    connection.player_data.character.service_account_id as u64,
                                    connection.player_data.character.content_id as u64,
                                    connection.player_data.character.actor_id,
                                    connection.player_data.character.name.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::RequestSearchInfo { content_id, .. } => {
                            let search_info;
                            {
                                let mut database = connection.database.lock();
                                let mut game_data = connection.game_data.lock();
                                // A party bot shown as a player has a made-up content ID and nothing in the
                                // database, which the lookup would panic on.
                                if database.find_character(Some(*content_id), None).is_none() {
                                    tracing::info!(
                                        "Search info asked for {content_id:#x}, no such character (a party bot?)"
                                    );
                                    search_info = None;
                                } else {
                                    search_info = Some(
                                        database
                                            .get_search_info(&mut game_data, *content_id as i64),
                                    );
                                }
                            }

                            if let Some(search_info) = search_info {
                                let ipc = ServerZoneIpcSegment::new(search_info);
                                connection.send_ipc_self(ipc).await;
                            }
                        }
                        ClientZoneIpcData::RequestAdventurerPlate { actor_id, .. } => {
                            let ipc;
                            {
                                let mut database = connection.database.lock();
                                let mut game_data = connection.game_data.lock();
                                // As above: a party bot shown as a player has no character to look up.
                                if database.has_character_with_actor_id(*actor_id) {
                                    ipc = Some(
                                        database.lookup_adventurer_plate(&mut game_data, *actor_id),
                                    );
                                } else {
                                    tracing::info!(
                                        "Adventurer plate asked for {actor_id}, no such character (a party bot?)"
                                    );
                                    ipc = None;
                                }
                            }
                            if let Some(ipc) = ipc {
                                connection.send_ipc_self(ipc).await;
                            }
                        }
                        ClientZoneIpcData::SearchPlayers {
                            classjobs,
                            minimum_level,
                            maximum_level,
                            grand_company,
                            languages,
                            online_status,
                            areas,
                            name,
                            ..
                        } => {
                            connection
                                .search_players(
                                    *classjobs,
                                    *minimum_level,
                                    *maximum_level,
                                    *grand_company,
                                    *languages,
                                    *online_status,
                                    *areas,
                                    name.clone(),
                                )
                                .await;
                        }
                        ClientZoneIpcData::EditSearchInfo(search_info) => {
                            connection.player_data.search_info.online_status = search_info
                                .online_status
                                .mask()
                                .last()
                                .copied()
                                .unwrap_or(OnlineStatus::Online);
                            connection.player_data.search_info.comment =
                                search_info.comment.clone();
                            connection.player_data.search_info.selected_languages =
                                search_info.selected_languages;
                            {
                                let mut database = connection.database.lock();
                                database.commit_search_info(&connection.player_data);
                            }
                            connection.update_online_status().await;
                        }
                        ClientZoneIpcData::RequestOwnSearchInfo { .. } => {
                            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::SetSearchInfo(
                                SearchInfo {
                                    online_status: OnlineStatusMask::from_online_status(
                                        connection.player_data.search_info.online_status,
                                    ),
                                    comment: connection.player_data.search_info.comment.clone(),
                                    selected_languages: connection
                                        .player_data
                                        .search_info
                                        .selected_languages,
                                    ..Default::default()
                                },
                            ));
                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::EnterTerritoryEvent { handler_id } => {
                            connection
                                .handle
                                .send(ToServer::EnterTerritoryEvent(
                                    connection.player_data.character.actor_id,
                                ))
                                .await;

                            connection
                                .start_event(
                                    ObjectTypeId {
                                        object_id: connection.player_data.character.actor_id,
                                        object_type: ObjectTypeKind::None,
                                    },
                                    *handler_id,
                                    EventType::EnterTerritory,
                                    connection.player_data.volatile.zone_id as u32,
                                    events,
                                    lua_player,
                                )
                                .await;
                            if let Some(event) = events.last_mut() {
                                event.0.on_enter_territory(&event.1, lua_player).await;
                            } else {
                                tracing::warn!(
                                    "No event started from EnterTerritoryEvent?! This probably means its missing a handler for its type."
                                );
                            }
                        }
                        ClientZoneIpcData::Trade { sequence, .. } => {
                            // TODO: This needs a lot more research, but it's good enough for now to act as a stub to prevent client softlocks
                            // When trading, the client sends the trade opcode twice for unknown (at this time) reasons, so we need to keep track of where we're at in the sequence
                            let unk1;
                            if !connection.is_trading {
                                tracing::info!("Trading is unimplemented");
                                unk1 = 22; // Some ack to let the client proceed with the trade sequence
                                connection.is_trading = true;
                            } else {
                                unk1 = 2; // Some ack to tell the client that the target can't trade at this time
                                connection.is_trading = false;
                            }

                            connection.send_inventory_ack(*sequence, 7, unk1).await;
                        }
                        ClientZoneIpcData::ShareStrategyBoard {
                            content_id,
                            board_data,
                        } => {
                            tracing::info!(
                                "{} is sharing a strategy board with their party!",
                                connection.player_data.character.actor_id
                            );
                            connection
                                .handle
                                .send(ToServer::ShareStrategyBoard(
                                    connection.player_data.character.actor_id,
                                    connection.player_data.character.content_id as u64,
                                    connection.party_id,
                                    *content_id,
                                    board_data.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::StrategyBoardReceived { content_id, .. } => {
                            tracing::info!(
                                "{} has received a strategy board from another player in their party!",
                                connection.player_data.character.actor_id
                            );
                            connection
                                .handle
                                .send(ToServer::StrategyBoardReceived(
                                    connection.party_id,
                                    connection.player_data.character.content_id as u64,
                                    *content_id,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::StrategyBoardUpdate(update_data) => {
                            // No logging here due to how spammy it is since it sends an update every frame or so while the object is moving.
                            connection
                                .handle
                                .send(ToServer::StrategyBoardRealtimeUpdate(
                                    connection.player_data.character.actor_id,
                                    connection.player_data.character.content_id as u64,
                                    connection.party_id,
                                    update_data.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::RealtimeStrategyBoardFinished { .. } => {
                            tracing::info!(
                                "{} is finished sharing their strategy board in realtime!",
                                connection.player_data.character.actor_id
                            );
                            connection
                                .handle
                                .send(ToServer::StrategyBoardRealtimeFinished(connection.party_id))
                                .await;
                        }
                        ClientZoneIpcData::ApplyFieldMarkerPreset(waymark_preset) => {
                            connection
                                .handle
                                .send(ToServer::ApplyWaymarkPreset(
                                    connection.player_data.character.actor_id,
                                    connection.party_id,
                                    *waymark_preset,
                                    connection.player_data.volatile.zone_id,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::RequestFreeCompanyShortMessage { .. } => {
                            tracing::warn!(
                                "Requesting a free company short message is unimplemented"
                            );
                        }
                        ClientZoneIpcData::InitiateReadyCheck { .. } => {
                            // TODO: Remove this `let party_id` in an upcoming party refactor, this is temporary
                            let party_id = if connection.party_id != 0 {
                                Some(connection.party_id)
                            } else {
                                None
                            };

                            connection
                                .handle
                                .send(ToServer::ReadyCheckInitiated(
                                    party_id,
                                    connection.player_data.character.actor_id,
                                    connection.player_data.character.service_account_id as u64,
                                    connection.player_data.character.content_id as u64,
                                    connection.player_data.character.name.clone(),
                                ))
                                .await;
                        }
                        ClientZoneIpcData::ReadyCheckResponse { response } => {
                            // TODO: Remove this `let party_id` in an upcoming party refactor, this is temporary
                            let party_id = if connection.party_id != 0 {
                                Some(connection.party_id)
                            } else {
                                None
                            };

                            // As usual, another client value that doesn't match up with values we respond with...
                            let response = match response {
                                1 => ReadyCheckReply::Yes,
                                _ => ReadyCheckReply::No, // This opcode should only ever give us 1 or 0, but it doesn't hurt to guard against bad inputs.
                            };

                            tracing::info!(
                                "client {} replied to ready check with {response:#?}",
                                connection.player_data.character.actor_id
                            );

                            connection
                                .handle
                                .send(ToServer::ReadyCheckResponse(
                                    party_id,
                                    connection.player_data.character.actor_id,
                                    connection.player_data.character.service_account_id as u64,
                                    connection.player_data.character.content_id as u64,
                                    connection.player_data.character.name.clone(),
                                    response,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::RequestMarketBoardItems {
                            start_index,
                            request_id,
                            kind,
                            category,
                            flags,
                            unk,
                        } => {
                            let filter = kawari_world::marketboard::search_filter(*flags, unk);
                            connection
                                .marketboard_search(
                                    *start_index,
                                    *request_id,
                                    *kind,
                                    *category,
                                    filter,
                                )
                                .await;
                        }
                        ClientZoneIpcData::RequestPlayerRetainerInfo { .. } => {
                            connection.send_player_retainer_info().await;
                        }
                        ClientZoneIpcData::RequestRetainerSaleHistory { retainer_id } => {
                            connection.send_retainer_sale_history(*retainer_id).await;
                        }
                        ClientZoneIpcData::SaveGlamourPlate {
                            plate,
                            changed,
                            what,
                            indices,
                            dye0_containers,
                            dye0_slots,
                            dye1_containers,
                            dye1_slots,
                            stain0,
                            stain1,
                        } => {
                            let slots = std::array::from_fn(|slot| {
                                kawari_world::glamour::PlateSlotChange {
                                    changed: changed[slot] != 0,
                                    what: what[slot],
                                    index: indices[slot],
                                    stain0: stain0[slot],
                                    stain1: stain1[slot],
                                }
                            });
                            let dyes = std::array::from_fn(|slot| {
                                [
                                    (dye0_containers[slot], dye0_slots[slot]),
                                    (dye1_containers[slot], dye1_slots[slot]),
                                ]
                            });
                            connection.save_glamour_plate(*plate, &slots, &dyes).await;
                        }
                        ClientZoneIpcData::MarketBoardOpened { request_id, .. } => {
                            connection.marketboard_opened(*request_id).await;
                        }
                        ClientZoneIpcData::SetFriendGroupIcon(icon_info) => {
                            connection.set_friend_group_icon(icon_info).await;
                        }
                        ClientZoneIpcData::SetRetainerAppearance { voice, look, .. } => {
                            connection.pending_retainer_appearance = Some((*voice, look.clone()));
                        }
                        ClientZoneIpcData::CreateLocalLinkshellRequest { data } => {
                            // On 2026.10.06 this is also an event's text answer (a retainer's name at the vocate.)
                            if let Some(text) = kawari::ipc::zone::event_text(data)
                                && let Some(event) = events.last()
                            {
                                connection.pending_event_text = Some(text.text.clone());
                                event
                                    .0
                                    .on_yield(
                                        &event.1,
                                        connection,
                                        text.scene,
                                        text.yield_id,
                                        &[],
                                        lua_player,
                                    )
                                    .await;
                            } else {
                                tracing::warn!("Creating local linkshells is unimplemented");
                            }
                        }
                        // An event's text answer (a chocobo's name), in the size its text needs.
                        ClientZoneIpcData::EventYieldText16 { data: _ }
                        | ClientZoneIpcData::EventYieldText24 { data: _ } => {
                            let text = match &data.data {
                                ClientZoneIpcData::EventYieldText16 { data } => {
                                    kawari::ipc::zone::event_text_short(data)
                                }
                                ClientZoneIpcData::EventYieldText24 { data } => {
                                    kawari::ipc::zone::event_text_short(data)
                                }
                                _ => None,
                            };
                            if let Some(text) = text
                                && let Some(event) = events.last()
                            {
                                connection.pending_event_text = Some(text.text.clone());
                                event
                                    .0
                                    .on_yield(
                                        &event.1,
                                        connection,
                                        text.scene,
                                        text.yield_id,
                                        &[],
                                        lua_player,
                                    )
                                    .await;
                            }
                        }
                        ClientZoneIpcData::CrossworldLinkshellMemberListRequest {
                            linkshell_id,
                            sequence,
                        } => {
                            connection
                                .send_cwlinkshell_members(*linkshell_id, *sequence)
                                .await;
                        }
                        ClientZoneIpcData::OpenTreasure { entity_id } => {
                            connection
                                .handle
                                .send(ToServer::OpenTreasure(
                                    connection.player_data.character.actor_id,
                                    *entity_id,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::LootRoll {
                            treasure_id,
                            slot,
                            choice,
                            ..
                        } => {
                            connection
                                .handle
                                .send(ToServer::LootRoll(
                                    connection.player_data.character.actor_id,
                                    *treasure_id,
                                    *slot,
                                    *choice,
                                ))
                                .await;
                        }
                        // Air Force One's score as the ride ends (`zone_connection/ride_shooting.rs`.)
                        ClientZoneIpcData::RideShootingScore { score, hits, .. } => {
                            connection
                                .ride_shooting_score(*score, *hits, events, lua_player)
                                .await;
                        }
                        ClientZoneIpcData::CrossRealmListingsRequest1 { max_results, .. } => {
                            let results_aligned = max_results.div_ceil(4) * 4; // each packet holds 4
                            let required_packets = results_aligned / 4;
                            for i in 0..required_packets {
                                let ipc = ServerZoneIpcSegment::new(
                                    ServerZoneIpcData::CrossRealmListings(CrossRealmListings {
                                        unk10: 0,
                                        unk11: 0xFFFFFFFF,
                                        unk12: 0xFFFFFFFF,
                                        segment_index: if i + 1 == required_packets {
                                            0
                                        } else {
                                            i + 1
                                        },
                                        entries: vec![
                                            CrossRealmListing {
                                                listing_id: 1,
                                                account_id: 1,
                                                content_id: 1,
                                                category: 1,
                                                duty: 1,
                                                duty_type: 1,
                                                world_id: 1,
                                                objective: 1,
                                                beginners_welcome: 1,
                                                duty_finder_settings: 1,
                                                loot_rule: 1,
                                                last_patch_hotfix_timestamp: 1,
                                                time_left: 1,
                                                avg_item_lv: 1,
                                                home_world_id: 1,
                                                client_language: 1,
                                                total_slots: 1,
                                                slots_filled: 1,
                                                join_condition_flags: 1,
                                                is_alliance: 1,
                                                number_of_parties: 1,
                                                slot_flags: [1; 8],
                                                jobs_present: [1; 8],
                                                bad_padding: Vec::new(),
                                                recruiter_name: "Test Listing".to_string(),
                                                comment: "Riduculous Ties".to_string(),
                                                bad_padding2: Vec::new(),
                                            };
                                            4
                                        ],
                                    }),
                                );
                                connection.send_ipc_self(ipc).await;
                            }

                            // send overview
                            let ipc = ServerZoneIpcSegment::new(
                                ServerZoneIpcData::CrossRealmListingsOverview { unk: [0; 48] },
                            );
                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::CrossRealmListingsRequest2 { .. } => {
                            // NOTE: not sure what we need from here
                        }
                        ClientZoneIpcData::ViewCrossRealmListing { listing_id } => {
                            let ipc = ServerZoneIpcSegment::new(
                                ServerZoneIpcData::CrossRealmListingInformation {
                                    listing_id: *listing_id,
                                    unk: [0; 456],
                                },
                            );
                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::CheckCWLinkshellNameAvailability { name, .. } => {
                            connection
                                .check_cwlinkshell_name_availability(name.clone())
                                .await;
                        }
                        ClientZoneIpcData::CreateNewCrossworldLinkshell { name } => {
                            connection.create_crossworld_linkshell(name.clone()).await;
                        }
                        ClientZoneIpcData::LeaveCrossworldLinkshell { linkshell_id } => {
                            connection
                                .remove_linkshell_member(
                                    *linkshell_id,
                                    connection.player_data.character.content_id as u64,
                                    CWLSLeaveReason::Leaving,
                                )
                                .await;
                        }
                        ClientZoneIpcData::DisbandCrossworldLinkshell { linkshell_id } => {
                            connection.disband_linkshell(*linkshell_id).await;
                        }
                        ClientZoneIpcData::RenameCrossworldLinkshell { linkshell_id, name } => {
                            connection
                                .rename_linkshell(*linkshell_id, name.clone())
                                .await;
                        }
                        ClientZoneIpcData::SetCWLSMemberRank {
                            linkshell_id,
                            content_id,
                            rank,
                            ..
                        } => {
                            connection
                                .set_linkshell_rank(*linkshell_id, *content_id, *rank)
                                .await;
                        }
                        ClientZoneIpcData::RemoveCWLSMember {
                            linkshell_id,
                            content_id,
                        } => {
                            connection
                                .remove_linkshell_member(
                                    *linkshell_id,
                                    *content_id,
                                    CWLSLeaveReason::Kicked,
                                )
                                .await;
                        }
                        ClientZoneIpcData::InviteCharacterToCWLS {
                            linkshell_id,
                            content_id,
                        } => {
                            if let Some(result) = connection
                                .invite_to_linkshell(*content_id, *linkshell_id)
                                .await
                            {
                                connection.send_linkshell_error(result).await;
                            }
                        }
                        ClientZoneIpcData::LinkshellInviteReply {
                            linkshell_id,
                            response,
                        } => {
                            // Guard against bogus replies by checking if the client is in the shell or not. Invitees are considered actual members, but they just can't receive chat messages.
                            if connection.is_in_linkshell(*linkshell_id).await {
                                match response {
                                    LinkshellInviteResponse::Accepted => {
                                        connection.accepted_linkshell_invite(*linkshell_id).await
                                    }
                                    LinkshellInviteResponse::Declined => {
                                        connection
                                            .remove_linkshell_member(
                                                *linkshell_id,
                                                connection.player_data.character.content_id as u64,
                                                CWLSLeaveReason::DeclinedInvite,
                                            )
                                            .await
                                    }
                                }
                            } else {
                                connection
                                    .send_linkshell_error(LogMessageType::UnableToAcceptLSInvite)
                                    .await;
                            }
                        }
                        ClientZoneIpcData::RequestMailbox { unk1, .. } => {
                            connection.send_letter_previews(*unk1).await;
                        }
                        ClientZoneIpcData::SendLetter {
                            recipient_content_id,
                            attached_items,
                            message,
                        } => {
                            connection
                                .send_letter(
                                    *recipient_content_id,
                                    *attached_items,
                                    message.clone(),
                                )
                                .await;
                        }
                        ClientZoneIpcData::ViewLetter {
                            sender_content_id,
                            timestamp,
                            ..
                        } => {
                            connection.view_letter(*sender_content_id, *timestamp).await;
                        }
                        ClientZoneIpcData::DeleteLetter {
                            sender_content_id,
                            timestamp,
                            ..
                        } => {
                            connection
                                .delete_letter(*sender_content_id, *timestamp)
                                .await;
                        }
                        ClientZoneIpcData::RewardDeliveryRequest { .. } => {
                            tracing::info!("Requesting reward delivery items is unimplemented");
                        }
                        ClientZoneIpcData::TakeLetterAttachments {
                            sender_content_id,
                            timestamp,
                            ..
                        } => {
                            connection
                                .take_attachments_from_letter(*sender_content_id, *timestamp)
                                .await;
                        }
                        ClientZoneIpcData::RemoveFriend {
                            content_id, name, ..
                        } => {
                            connection
                                .remove_from_friend_list(*content_id, name.clone())
                                .await;
                        }
                        ClientZoneIpcData::Dive {
                            target_position,
                            rotation,
                            ..
                        } => {
                            connection
                                .change_zone(
                                    connection.player_data.volatile.zone_id as u16,
                                    Some(*target_position),
                                    Some(*rotation),
                                    WarpType::Dive,
                                    474,
                                )
                                .await;
                        }
                        ClientZoneIpcData::WorldInteraction {
                            action,
                            param1,
                            param2,
                            param3,
                            param4,
                            position,
                        } => {
                            match action {
                                0xD1 => {
                                    // Underwater portal
                                    connection
                                        .handle
                                        .send(ToServer::EnterZoneJump(
                                            connection.id,
                                            connection.player_data.character.actor_id,
                                            *param1,
                                            WarpType::Event,
                                            526,
                                        ))
                                        .await;
                                }
                                0x25F => {
                                    // Surfacing from diving
                                    connection
                                        .change_zone(
                                            connection.player_data.volatile.zone_id as u16,
                                            Some(*position),
                                            Some(connection.player_data.volatile.rotation as f32),
                                            WarpType::Dive,
                                            483,
                                        )
                                        .await;
                                }
                                _ => {
                                    tracing::info!(
                                        "Client executed unimplemented world interaction {action} with params {param1} {param2} {param3} {param4}"
                                    );
                                }
                            }
                        }
                        ClientZoneIpcData::PlaceFurniture {
                            container,
                            slot,
                            position,
                            rotation,
                            spawn_furniture,
                            plot_index,
                            ..
                        } => {
                            let intended_use = lua_player.zone_data.intended_use;

                            // TODO: Also reject if they're not the owner/shared tenant
                            if !in_housing_area(intended_use) {
                                tracing::error!(
                                    "The player attempted to place furniture while not within a housing zone! Rejecting request!"
                                );
                                continue;
                            }

                            tracing::info!(
                                "Client placed furniture! {:#?} {:#?} {:#?} {:#?} {:#?}",
                                container,
                                slot,
                                position,
                                rotation,
                                spawn_furniture,
                            );

                            let transfer_item = if let Some(the_item) =
                                connection.player_data.inventory.get_item(*container, *slot)
                            {
                                the_item
                            } else if let Some(the_item) = connection
                                .player_data
                                .house_inventory
                                .get_item(*container, *slot)
                            {
                                the_item
                            } else {
                                tracing::error!(
                                    "The client attempted to use an invalid container to place a furniture item! Rejecting request!"
                                );
                                continue;
                            };

                            let item_id;
                            {
                                let mut game_data = connection.game_data.lock();
                                let result = game_data.get_furniture_item_id(transfer_item.item_id);
                                item_id = result.unwrap_or_default();
                            }

                            let desired_pages = connection
                                .player_data
                                .house_inventory
                                .get_desired_pages_from_intendeduse(intended_use, !spawn_furniture);

                            let Some(result) = connection
                                .player_data
                                .house_inventory
                                .add_in_empty_slot(transfer_item, desired_pages)
                            else {
                                tracing::error!(
                                    "Unable to add item to this housing inventory, it's full!"
                                );
                                continue;
                            };

                            // Next, remove the item from the player's main inventory or the storeroom.
                            {
                                let old_slot = if let Some(the_item) = connection
                                    .player_data
                                    .inventory
                                    .get_item_mut(*container, *slot)
                                {
                                    the_item
                                } else if let Some(the_item) = connection
                                    .player_data
                                    .house_inventory
                                    .get_item_mut(*container, *slot)
                                {
                                    the_item
                                } else {
                                    continue; // This shouldn't even be reachable or possible but it's not overly desirable to crash intentionally, so we'll just skip over it.
                                };

                                *old_slot = Item::default();

                                let ipc = ServerZoneIpcSegment::new(
                                    ServerZoneIpcData::UpdateInventorySlot(ItemInfo {
                                        sequence: 0,
                                        container: *container,
                                        slot: *slot,
                                        ..(Item::default()).into()
                                    }),
                                );
                                connection.send_ipc_self(ipc).await;
                            }

                            // Next, update the client's inventory.
                            let src_container_type = container;
                            let dst_container_type = result.container;
                            connection
                                .send_affected_containers(*src_container_type, dst_container_type)
                                .await;

                            // If the client opted to move furniture to the storeroom, there's nothing further to do here.
                            if !spawn_furniture {
                                continue;
                            }

                            // Finally, acknowledge the placement.
                            // TODO: We need to store the coordinates when things are persistent
                            let indoors = intended_use == TerritoryIntendedUse::HousingIndoor;
                            // TODO: implement dyes...
                            let stain = 0;

                            connection
                                .handle
                                .send(ToServer::PlaceFurniture(
                                    connection.player_data.character.actor_id,
                                    result.container,
                                    result.slot,
                                    item_id,
                                    stain,
                                    *position,
                                    indoors,
                                    *rotation,
                                    *plot_index,
                                ))
                                .await;

                            // This ack doesn't need to be networked
                            connection
                                .actor_control_self(ActorControlCategory::FurniturePlacedAck {
                                    unk1: 0,
                                    unk2: 0,
                                    unk3: 0,
                                    unk4: 0,
                                })
                                .await;
                        }
                        ClientZoneIpcData::TranslateFurniture {
                            house_id,
                            slot,
                            position,
                            rotation,
                            unk2,
                            unk3,
                        } => {
                            let intended_use = lua_player.zone_data.intended_use;

                            // TODO: Also reject if they're not the owner/shared tenant
                            if !in_housing_area(intended_use) {
                                tracing::warn!(
                                    "Client attempted to move furniture when not in a housing area! Rejecting request!"
                                );
                                continue;
                            }

                            tracing::info!(
                                "Client moved furniture! {:#?} {:#?}, {:#?}, {:#?} {:#?} {:#?}",
                                house_id,
                                slot,
                                position,
                                rotation,
                                unk2,
                                unk3
                            );

                            // TODO: We need to store the new coordinates and rotation when making everything persistent!
                            let indoors = intended_use == TerritoryIntendedUse::HousingIndoor;

                            // Determine which container the moved item belongs to.
                            // TODO: This will need to be expanded in 7.5
                            let storage_id;
                            if indoors {
                                if *slot < 50 {
                                    storage_id = ContainerType::HousingInteriorPlacedItems1;
                                } else if *slot < 100 {
                                    storage_id = ContainerType::HousingInteriorPlacedItems2;
                                } else if *slot < 150 {
                                    storage_id = ContainerType::HousingInteriorPlacedItems3;
                                } else if *slot < 200 {
                                    storage_id = ContainerType::HousingInteriorPlacedItems4;
                                } else if *slot < 250 {
                                    storage_id = ContainerType::HousingInteriorPlacedItems5;
                                } else if *slot < 300 {
                                    storage_id = ContainerType::HousingInteriorPlacedItems6;
                                } else if *slot < 350 {
                                    storage_id = ContainerType::HousingInteriorPlacedItems7;
                                } else if *slot < 400 {
                                    storage_id = ContainerType::HousingInteriorPlacedItems8;
                                } else {
                                    tracing::warn!(
                                        "Client tried to move furniture beyond the bounds of current housing limits! Rejecting request!"
                                    );
                                    continue;
                                }
                            } else {
                                if *slot < 50 {
                                    storage_id = ContainerType::HousingExteriorPlacedItems;
                                } else {
                                    tracing::warn!(
                                        "Client tried to move furniture beyond the bounds of current housing limits! Rejecting request!"
                                    );
                                    continue;
                                }
                            };

                            connection
                                .handle
                                .send(ToServer::TranslateFurniture(
                                    connection.player_data.character.actor_id,
                                    // TODO: Maybe revise sending this tuple, we'll see
                                    (
                                        house_id.unit.apartment_flag,
                                        house_id.unit.apartment_division_plot_index,
                                    ),
                                    *slot,
                                    *position,
                                    *rotation,
                                    indoors,
                                ))
                                .await;

                            // This ack doesn't need to be networked
                            connection
                                .actor_control_self(ActorControlCategory::FurnitureTranslatedAck {
                                    storage_id,
                                    slot: (*slot % MAX_LARGE_STORAGE as u16) as u32,
                                    plot_number: if !house_id.unit.apartment_flag {
                                        house_id.unit.apartment_division_plot_index as u16
                                    } else {
                                        0
                                    },
                                })
                                .await;
                        }
                        ClientZoneIpcData::UpdatePositionHandlerInstance {
                            rotation,
                            anim_type,
                            anim_state,
                            jump_state,
                            position,
                            ..
                        } => {
                            // NOTE: Keep in sync with the UpdatePositionHandler.
                            if connection.spawned_in {
                                connection.player_data.volatile.rotation = *rotation as f64;
                                connection.player_data.volatile.position = *position;

                                let party_id = if connection.party_id != 0 {
                                    Some(connection.party_id)
                                } else {
                                    None
                                };

                                connection
                                    .handle
                                    .send(ToServer::ActorMoved(
                                        connection.player_data.character.actor_id,
                                        *position,
                                        *rotation,
                                        *anim_type,
                                        *anim_state,
                                        *jump_state,
                                        party_id,
                                    ))
                                    .await;
                            }
                        }
                        ClientZoneIpcData::FallFromArena { .. } => {
                            tracing::info!("Player fell from arena!");
                            // Signal to the global server to kill us.
                            connection
                                .handle
                                .send(ToServer::Kill(
                                    connection.id,
                                    connection.player_data.character.actor_id,
                                ))
                                .await;
                        }
                        ClientZoneIpcData::DyeInformation(dye_information) => {
                            connection.dyeing_information = Some(dye_information.clone());
                        }
                        ClientZoneIpcData::RequestRecruitingPartyCount { .. } => {
                            // Send an empty packet so it's at least says zero!
                            let ipc = ServerZoneIpcSegment::new(
                                ServerZoneIpcData::RecruitingPartyCount { unk1: [0; 968] },
                            );
                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::UnkMarketBoardRequest1 { item_id, .. } => {
                            connection.marketboard_open(*item_id).await;
                        }
                        ClientZoneIpcData::UnkMarketBoardRequest2 {
                            start_index,
                            request_id,
                        } => {
                            connection.marketboard_page(*start_index, *request_id).await;
                        }
                        ClientZoneIpcData::MarketBoardPurchaseHandler {
                            listing_id,
                            item_id,
                            quantity,
                            price_per_unit,
                            ..
                        } => {
                            connection
                                .marketboard_purchase(
                                    *listing_id,
                                    *item_id,
                                    *quantity,
                                    *price_per_unit,
                                )
                                .await;
                        }
                        ClientZoneIpcData::PerformanceNote { .. } => {
                            // TODO: what to do with these?
                        }
                        ClientZoneIpcData::CreateFellowship { .. } => {
                            let ipc =
                                ServerZoneIpcSegment::new(ServerZoneIpcData::ShowLinkshellError {
                                    log_message: LogMessageType::UnableToCreateMoreFellowships
                                        as u16,
                                    unk: 0, // I don't have a capture of this yet, but zero works fine.
                                });

                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::SearchFellowshipsJoin { .. } => {
                            let ipc =
                                ServerZoneIpcSegment::new(ServerZoneIpcData::ShowLinkshellError {
                                    log_message: LogMessageType::UnableToJoinMoreFellowships as u16,
                                    unk: 0x0124_0E82, // Observed in a capture, but it doesn't seem to matter what we set it to, zero works too.
                                });

                            connection.send_ipc_self(ipc).await;
                        }
                        ClientZoneIpcData::Unknown { unk } => {
                            tracing::warn!(
                                "Unknown Zone packet {:?} recieved ({} bytes)",
                                data.header.op_code,
                                unk.len()
                            );
                            kawari_world::fork_opcodes::hint_unknown_packet(
                                "ClientZoneIpcType",
                                kawari::packet::PredefinedOpcode::get_opcode(&data.header.op_code),
                                unk.len(),
                            );
                        }
                    }
                }
                SegmentData::KeepAliveRequest { id, timestamp } => {
                    connection.send_keep_alive(*id, *timestamp).await
                }
                SegmentData::KeepAliveResponse { .. } => {
                    // these should be safe to ignore
                }
                _ => {
                    panic!(
                        "ZoneConnection: The server is recieving a response or unknown packet: {segment:#?}"
                    )
                }
            }
        }

        // Process any queued packets from scripts and whatnot
        lua_player.queued_tasks.append(&mut connection.queued_tasks);
        if connection.process_lua_player(lua_player, events).await {
            // If requested to run again (currently relevant for finishing events) then do so.
            connection.process_lua_player(lua_player, events).await;
        }

        // Update lua player
        lua_player.player_data = connection.player_data.clone();
    }

    true
}

/// Process internal server messages.
async fn process_server_msg(
    connection: &mut ZoneConnection,
    lua_player: &mut LuaPlayer,
    events: &mut Vec<(Box<dyn EventHandler>, Event)>,
    client_handle: ClientHandle,
    msg: Option<FromServer>,
) {
    if let Some(msg) = msg {
        match msg {
            FromServer::Message(msg) => connection.send_message(msg).await,
            FromServer::ActorSpawn(actor, spawn) => connection.spawn_actor(actor, spawn).await,
            FromServer::ActorMove(
                actor_id,
                position,
                rotation,
                anim_type,
                anim_state,
                jump_state,
                speed,
            ) => {
                connection
                    .set_actor_position(
                        actor_id, position, rotation, anim_type, anim_state, jump_state, speed,
                    )
                    .await
            }
            FromServer::DeleteActor(object_id, spawn_index) => {
                connection.delete_actor(object_id, spawn_index).await
            }
            FromServer::DeleteObject(spawn_index) => connection.delete_object(spawn_index).await,
            FromServer::ActorControl(actor_id, actor_control) => {
                connection.actor_control(actor_id, actor_control).await
            }
            FromServer::ActorControlTarget(actor_id, target, actor_control) => {
                connection
                    .actor_control_target(actor_id, target, actor_control)
                    .await
            }
            FromServer::ActorControlSelf(actor_control) => {
                connection.actor_control_self(actor_control).await
            }
            FromServer::ActorSummonsMinion(minion_id) => {
                connection
                    .handle
                    .send(ToServer::ActorSummonsMinion(
                        connection.player_data.character.actor_id,
                        minion_id,
                    ))
                    .await;
                connection.active_minion = minion_id;
            }
            FromServer::ActorDespawnsMinion => {
                connection
                    .handle
                    .send(ToServer::ActorDespawnsMinion(
                        connection.player_data.character.actor_id,
                    ))
                    .await;
                connection.active_minion = 0;
            }
            FromServer::ActorEquip(
                actor_id,
                main_weapon_id,
                sub_weapon_id,
                model_ids,
                second_model_stain_ids,
            ) => {
                connection
                    .update_equip(
                        actor_id,
                        main_weapon_id,
                        sub_weapon_id,
                        model_ids,
                        second_model_stain_ids,
                    )
                    .await
            }
            FromServer::LoseEffect(effect_id, effect_param, effect_source_actor_id) => {
                connection
                    .lose_effect(effect_id, effect_param, effect_source_actor_id)
                    .await
            }
            FromServer::Conditions(conditions) => {
                connection.conditions = conditions;
                connection.send_conditions().await;
            }
            FromServer::ToggleCondition(condition, set) => {
                if connection.conditions.has_condition(condition) != set {
                    connection.conditions.toggle_condition(condition, set);
                    connection.send_conditions().await;
                }
            }
            FromServer::ChangeZone(
                zone_id,
                content_finder_condition_id,
                weather_id,
                position,
                rotation,
                lua_zone,
                handler_id,
            ) => {
                connection
                    .handle_zone_change(
                        zone_id,
                        content_finder_condition_id,
                        weather_id,
                        position,
                        rotation,
                        &lua_zone,
                        handler_id,
                    )
                    .await;
                lua_player.zone_data = lua_zone;
            }
            FromServer::TeleportOffered(party_member_index, teleport_info) => {
                // By default, don't allow the player to go.
                let mut ineligible_for_teleport = true;
                let aetheryte_id = teleport_info.aetheryte_id as u32;

                // If the player has this destination unlocked and has no previously offered teleport, they can go too.
                let unlocked = connection
                    .player_data
                    .aetheryte
                    .unlocked
                    .contains(aetheryte_id);
                if unlocked && connection.offered_teleport.is_none() {
                    connection.offered_teleport = Some(teleport_info);
                    ineligible_for_teleport = false;
                }

                connection
                    .actor_control_self(ActorControlCategory::TeleportOffered {
                        ineligible_for_teleport,
                        aetheryte_id,
                        party_member_index,
                    })
                    .await;
            }
            FromServer::SocialInvite(
                sender_account_id,
                sender_content_id,
                sender_name,
                invite_type,
            ) => {
                connection
                    .received_social_invite(
                        sender_account_id,
                        sender_content_id,
                        sender_name,
                        invite_type,
                    )
                    .await;
            }
            FromServer::InvitationResult(
                sender_account_id,
                sender_content_id,
                sender_name,
                invite_type,
                invite_reply,
            ) => {
                connection
                    .received_invitation_response(
                        sender_account_id,
                        sender_content_id,
                        sender_name,
                        invite_type,
                        invite_reply,
                    )
                    .await
            }
            FromServer::InvitationReplyResult(
                sender_account_id,
                sender_name,
                invite_type,
                invite_reply,
            ) => {
                connection
                    .send_invite_reply_result(
                        sender_account_id,
                        sender_name,
                        invite_type,
                        invite_reply,
                    )
                    .await
            }
            FromServer::PartyUpdate(targets, update_status, party_info) => {
                connection
                    .send_party_update(targets, update_status, party_info)
                    .await
            }
            FromServer::BotParty(members) => connection.send_bot_party(members).await,
            FromServer::BotPartyUpdate(members, update_status, targets) => {
                connection
                    .send_bot_party_update(members, update_status, targets)
                    .await
            }
            FromServer::BotInviteAccepted(account_id, content_id, name, invite_type) => {
                connection
                    .send_invite_update(
                        account_id,
                        content_id,
                        0,
                        invite_type,
                        None,
                        name,
                        Some(kawari::ipc::zone::InviteReply::Accepted),
                    )
                    .await
            }
            FromServer::InviteCharacterResult(
                content_id,
                message_id,
                invite_type,
                character_name,
            ) => {
                connection
                    .invite_character_result(
                        content_id,
                        message_id,
                        invite_type,
                        character_name.clone(),
                    )
                    .await
            }
            FromServer::RejoinPartyAfterDisconnect(party_id) => {
                connection.party_id = party_id;
                connection.rejoining_party = true;
            }
            FromServer::PacketSegment(ipc, from_actor_id) => {
                connection.send_ipc_from(from_actor_id, ipc).await;
            }
            FromServer::NewTasks(mut tasks) => {
                // Run them now (e.g. the warp from casting Teleport): left queued, they'd wait for the client's next
                // packet, which can be several seconds away when the player is standing still.
                lua_player.queued_tasks.append(&mut tasks);
                if connection.process_lua_player(lua_player, events).await {
                    connection.process_lua_player(lua_player, events).await;
                }
                lua_player.player_data = connection.player_data.clone();
            }
            FromServer::NewStatusEffects(status_effects) => {
                lua_player.status_effects = status_effects
            }
            FromServer::SpawnObject(object) => connection.spawn_object(object).await,
            FromServer::LocationDiscovered(map_id, map_part_id) => {
                connection.discover_location(map_id, map_part_id).await
            }
            FromServer::StrategyBoardShared(content_id, board_data) => {
                connection
                    .received_strategy_board(content_id, board_data)
                    .await
            }
            FromServer::StrategyBoardSharedAck(content_id) => {
                connection.strategy_board_ack(content_id).await
            }
            FromServer::StrategyBoardRealtimeUpdate(update_data) => {
                connection.strategy_board_updated(update_data).await
            }
            FromServer::StrategyBoardRealtimeFinished => {
                connection.strategy_board_realtime_finished().await
            }
            FromServer::WaymarkUpdated(id, placement_mode, position, zone_id) => {
                connection
                    .waymark_updated(id, placement_mode, position, zone_id)
                    .await
            }
            FromServer::WaymarkPreset(data, zone_id) => {
                connection.waymark_preset(data, zone_id).await
            }
            FromServer::EnteredInstanceEntranceRange(arg) => {
                tracing::info!("Showing leave duty dialog...");

                let object = ObjectTypeId {
                    object_id: connection.player_data.character.actor_id,
                    object_type: ObjectTypeKind::None,
                };
                let handler_id = HandlerId::new(HandlerType::GimmickRect, 1);

                connection
                    .start_event(
                        object,
                        handler_id,
                        EventType::WithinRange,
                        arg,
                        events,
                        lua_player,
                    )
                    .await;

                connection
                    .event_scene(
                        &events.last().unwrap().1,
                        2,
                        SceneFlags::NO_DEFAULT_CAMERA | SceneFlags::HIDE_HOTBAR,
                        Vec::new(),
                    )
                    .await;
            }
            FromServer::IncrementRestedExp => connection.add_rested_exp_seconds(10).await,
            FromServer::Countdown(account_id, content_id, name, starter_actor_id, duration) => {
                connection
                    .start_countdown(account_id, content_id, name, starter_actor_id, duration)
                    .await
            }
            FromServer::TargetSignToggled(sign_id, from_actor_id, target_actor) => {
                connection
                    .target_sign_toggled(sign_id, from_actor_id, target_actor)
                    .await
            }
            FromServer::DutyCleared(content_id, newcomer_here) => {
                connection.duty_cleared = Some(content_id);
                // Its first clear's EXP and the first-timer bonus (`ZoneConnection::duty_clear_rewards`.)
                connection
                    .duty_clear_rewards(content_id, newcomer_here)
                    .await;
                // Retail's wear and spiritbond came with the duty's completion (docs/src/fork/spiritbond.md.)
                connection.wear_gear_for_content(content_id).await;
                // A roulette's first clear of the day (`duty_finder::roulette_bonus`.)
                if let Some(roulette) = connection.entered_roulette.take() {
                    connection.roulette_cleared(roulette).await;
                }
            }
            FromServer::LeaveContent => {
                // Retail moved Sastasha's quest along as the player left the cleared duty (Bob's capture, 2026-10-23).
                if let Some(content_id) = connection.duty_cleared.take() {
                    connection
                        .on_duty_cleared(lua_player, events, content_id)
                        .await;
                }
                connection
                    .handle
                    .send(ToServer::LeaveContent(
                        connection.id,
                        connection.player_data.character.actor_id,
                        connection.old_zone_id,
                        connection.old_position,
                        connection.old_rotation,
                    ))
                    .await;
            }
            FromServer::FinishEvent => {
                connection.event_finish(events).await;
            }
            FromServer::BattleNpcKilled {
                actor_id,
                base_id,
                name_id,
                layout_id,
                level,
                gil,
            } => {
                connection
                    .on_battle_npc_killed(
                        lua_player, events, actor_id, base_id, name_id, layout_id, level,
                    )
                    .await;
                // Retail's gil came after the kill's EXP.
                if gil > 0 {
                    connection.obtain_dropped_gil(gil).await;
                }
            }
            FromServer::QuestEventActionInterrupted => {
                if let Some((_, event)) = events.last() {
                    tracing::info!("Event action interrupted during event {}", event.id);

                    // End the progress bar, and the event with it: the player can try again once they're clear.
                    connection
                        .actor_control_target(
                            connection.player_data.character.actor_id,
                            event.actor_id,
                            ActorControlCategory::EventAction {
                                unk1: 0,
                                id: connection.event_action_id,
                            },
                        )
                        .await;
                    // A nested event started for the action (the aetheryte's, attuning in a quest) ends too.
                    if let Some(handler_id) = connection.nested_event.take() {
                        connection
                            .send_ipc_self(ServerZoneIpcSegment::new(
                                ServerZoneIpcData::EventFinish {
                                    handler_id,
                                    event_type: EventType::Nest,
                                    result: 1,
                                    arg: 0,
                                },
                            ))
                            .await;
                        connection
                            .conditions
                            .remove_condition(kawari::ipc::zone::Condition::OccupiedInEvent);
                    }
                    connection.event_finish(events).await;
                }
            }
            FromServer::ItemUsedUp(item_id) => {
                connection.item_used_up(item_id).await;
            }
            FromServer::CompanionExp(exp) => {
                connection.companion_gained_exp(exp).await;
            }
            FromServer::EventItemUsed(item_id, target) => {
                // A key item's quest event (type EventItem, its argument the item), when its quest is underway.
                let quest = connection
                    .game_data
                    .lock()
                    .get_event_item(item_id)
                    .map(|info| info.quest)
                    .unwrap_or_default();
                if quest == 0 || !lua_player.has_active_quest(quest) {
                    tracing::info!("Event item {item_id} used, but quest {quest} isn't underway");
                } else if connection
                    .start_event(
                        target,
                        HandlerId(quest),
                        EventType::EventItem,
                        item_id,
                        events,
                        lua_player,
                    )
                    .await
                    && let Some(event) = events.last_mut()
                {
                    event
                        .0
                        .on_event_item(&event.1, item_id, target, lua_player)
                        .await;
                }
            }
            FromServer::ActionOnEventObject(action, target) => {
                // An action on an NPC or object a quest listens to for actions in its current sequence (the Quest
                // sheet's listeners of type ActionResult) starts that quest's event, its argument the action, as
                // retail did for Trial by Water's Cure on the wounded Wood Wailer (Bob's capture b23b91ef.)
                let base_id = lua_player
                    .zone_data
                    .cached_npc_base_ids
                    .get(&target.object_id)
                    .copied()
                    .or(lua_player
                        .zone_data
                        .cached_eobj_base_ids
                        .get(&target.object_id.0)
                        .copied());
                let quests: Vec<(u32, u8)> = lua_player
                    .player_data
                    .quest
                    .active
                    .0
                    .iter()
                    .map(|quest| (quest.id as u32 + 0x10000, quest.sequence))
                    .collect();
                let quest = base_id.and_then(|base_id| {
                    let mut game_data = connection.game_data.lock();
                    quests.into_iter().find_map(|(quest, sequence)| {
                        game_data
                            .quest_listens_for_action(quest, base_id, sequence)
                            .then_some(quest)
                    })
                });
                if events.is_empty()
                    && let Some(quest) = quest
                    && connection
                        .start_event(
                            target,
                            HandlerId(quest),
                            EventType::ActionResult,
                            action,
                            events,
                            lua_player,
                        )
                        .await
                    && let Some(event) = events.last_mut()
                {
                    event
                        .0
                        .on_action_result(&event.1, action, target, lua_player)
                        .await;
                }
            }
            FromServer::EmotedAt(emote, target) => {
                // An emote on an NPC or object a quest listens to in its current sequence (the Quest sheet's
                // listeners of type Emote) starts that quest's event, as on retail (Double Dealing, 466.)
                let base_id = lua_player
                    .zone_data
                    .cached_npc_base_ids
                    .get(&target.object_id)
                    .copied()
                    .or(lua_player
                        .zone_data
                        .cached_eobj_base_ids
                        .get(&target.object_id.0)
                        .copied());
                let quests: Vec<(u32, u8)> = lua_player
                    .player_data
                    .quest
                    .active
                    .0
                    .iter()
                    .map(|quest| (quest.id as u32 + 0x10000, quest.sequence))
                    .collect();
                let quest = base_id.and_then(|base_id| {
                    let mut game_data = connection.game_data.lock();
                    quests.into_iter().find_map(|(quest, sequence)| {
                        game_data
                            .quest_listens_for_emote(quest, base_id, sequence)
                            .then_some(quest)
                    })
                });
                if events.is_empty()
                    && let Some(quest) = quest
                    && connection
                        .start_event(
                            target,
                            HandlerId(quest),
                            EventType::Emote,
                            emote,
                            events,
                            lua_player,
                        )
                        .await
                    && let Some(event) = events.last_mut()
                {
                    event.0.on_emote(&event.1, emote, target, lua_player).await;
                }
            }
            FromServer::QuestEventActionCast => {
                if let Some((handler, event)) = events.last() {
                    tracing::info!("Event action finished casting during event {}", event.id);

                    // End the cast, otherwise the client stays busy and won't show anything else (like Sapphire does.)
                    connection
                        .actor_control_target(
                            connection.player_data.character.actor_id,
                            event.actor_id,
                            ActorControlCategory::EventAction {
                                unk1: 0,
                                id: connection.event_action_id,
                            },
                        )
                        .await;

                    handler.on_event_action_cast(event, lua_player).await;
                } else {
                    tracing::warn!(
                        "An event action finished casting, but there's no event anymore!"
                    );
                }
            }
            FromServer::QuestBattleComplete(
                handler_id,
                quest_id,
                sequence,
                stay,
                finish_scene,
                finish_flags,
                variables,
                fanfare,
                exit,
            ) => {
                // A win that sends them on elsewhere (the script's `EXIT_AFTER_WIN`): from now on that's where they
                // leave to (and where a logout puts them.)
                if let Some((zone_id, position, rotation)) = exit {
                    connection.old_zone_id = zone_id;
                    connection.old_position = position;
                    connection.old_rotation = rotation;
                }
                // The quest moves on straight away, the scenes are just the victory fanfare. Its variables change
                // in the same journal update, as retail sends it.
                let quest_id = connection.quest_for_quest_battle(handler_id.event_id(), quest_id);
                connection.preset_quest_variables(quest_id, &variables);
                connection.set_quest_sequence(quest_id, sequence).await;
                connection.quest_battle_stay = stay;
                connection.quest_battle_finish_scene = Some(finish_scene);
                connection.quest_battle_finish_flags = finish_flags;

                connection
                    .start_event(
                        ObjectTypeId {
                            object_id: connection.player_data.character.actor_id,
                            object_type: ObjectTypeKind::None,
                        },
                        handler_id,
                        EventType::GameProgress,
                        1,
                        events,
                        lua_player,
                    )
                    .await;
                if events.last_mut().is_some() {
                    if fanfare {
                        lua_player.play_scene(
                            QUEST_BATTLE_SCENE_COMPLETE,
                            SceneFlags::from_bits_retain(0x40000),
                            Vec::new(),
                        );
                    } else if stay {
                        // No fanfare and staying: nothing to play until the director sends them back.
                        lua_player.finish_event();
                    } else {
                        // Straight into the finish scene (ContentEventHandler sends them back when it ends.)
                        lua_player.play_scene(
                            finish_scene,
                            SceneFlags::from_bits_retain(finish_flags),
                            Vec::new(),
                        );
                    }
                }
            }
            FromServer::LeaveQuestBattle(handler_id) => {
                // The last scene, after which they're sent back (see ContentEventHandler.)
                connection.quest_battle_stay = false;
                let finish_scene = *connection
                    .quest_battle_finish_scene
                    .get_or_insert(QUEST_BATTLE_SCENE_FINISH);
                connection
                    .start_event(
                        ObjectTypeId {
                            object_id: connection.player_data.character.actor_id,
                            object_type: ObjectTypeKind::None,
                        },
                        handler_id,
                        EventType::GameProgress,
                        1,
                        events,
                        lua_player,
                    )
                    .await;
                if events.last_mut().is_some() {
                    lua_player.play_scene(
                        finish_scene,
                        SceneFlags::from_bits_retain(connection.quest_battle_finish_flags),
                        Vec::new(),
                    );
                }
            }
            FromServer::QuestBattleFailed => {
                // Zoning back in gives them a fresh spawn, which also brings them back to life.
                connection
                    .handle
                    .send(ToServer::LeaveContent(
                        connection.id,
                        connection.player_data.character.actor_id,
                        connection.old_zone_id,
                        connection.old_position,
                        connection.old_rotation,
                    ))
                    .await;
            }
            FromServer::FishBite => {
                let handler_id = HandlerId::new(HandlerType::Fishing, 1).0;

                let ipc = EventLogMessage {
                    handler_id: HandlerId(handler_id),
                    message_type: LogMessageType::FishingNoBite as u32,
                    ..Default::default()
                }
                .package()
                .unwrap();
                connection.send_ipc_self(ipc).await;

                connection
                    .event_scene(
                        &events.last().unwrap().1,
                        4,
                        SceneFlags::NO_DEFAULT_CAMERA,
                        vec![271, 0, 0],
                    )
                    .await;
            }
            FromServer::ActorDismounted(from_actor_id) => {
                // SetMode seems unnecessary (the dismount sequence works without it) but it's included for accuracy.
                connection
                    .set_character_mode(CharacterMode::Normal, 0, true)
                    .await;
                connection.player_data.volatile.current_mount = 0;
                connection
                    .actor_control(
                        from_actor_id,
                        ActorControlCategory::PlayDismountAnimation {
                            unk1: 0,
                            unk2: 0,
                            unk3: 0,
                        },
                    )
                    .await;

                connection
                    .actor_control(
                        from_actor_id,
                        ActorControlCategory::RidePillion {
                            target_actor_id: ObjectId::default(),
                            target_seat_index: 0,
                        },
                    )
                    .await;
            }
            FromServer::PartyMemberPositionsUpdate(positions) => {
                let ipc =
                    ServerZoneIpcSegment::new(ServerZoneIpcData::PartyMemberPositions(positions));
                connection.send_ipc_self(ipc).await;
            }
            FromServer::CommitParties(parties) => {
                let mut database = connection.database.lock();
                database.commit_parties(parties);
            }
            FromServer::TreasureSpawn(treasure) => connection.spawn_treasure(treasure).await,
            FromServer::LootWon(item_id) => connection.obtain_loot(item_id).await,
            FromServer::PersonalLoot(item_id) => connection.obtain_personal_loot(item_id).await,
            FromServer::LinkshellDisbanded(linkshell_id, linkshell_name) => {
                connection
                    .crossworld_linkshell_disbanded(linkshell_id, linkshell_name)
                    .await;
            }
            FromServer::LinkshellLeft(
                _,
                execute_content_id,
                target_content_id,
                target_name,
                reason_for_leaving,
                linkshell_id,
            ) => {
                connection
                    .member_left_linkshell(
                        execute_content_id,
                        target_content_id,
                        target_name.clone(),
                        reason_for_leaving,
                        linkshell_id,
                    )
                    .await
            }
            FromServer::LinkshellRenamed(
                from_content_id,
                from_name,
                linkshell_id,
                linkshell_name,
            ) => {
                connection
                    .linkshell_renamed(from_content_id, from_name, linkshell_id, linkshell_name)
                    .await
            }
            FromServer::LinkshellRankChanged(
                linkshell_id,
                from_content_id,
                target_content_id,
                rank,
                target_name,
            ) => {
                connection
                    .linkshell_rank_set(
                        linkshell_id,
                        from_content_id,
                        target_content_id,
                        rank,
                        target_name,
                    )
                    .await;
            }
            FromServer::LinkshellInviteReceived(invite) => {
                connection.received_linkshell_invite(invite).await;
            }
            FromServer::LinkshellInviteAccepted(
                linkshell_id,
                from_content_id,
                from_name,
                linkshell_name,
            ) => {
                connection
                    .member_joined_linkshell(
                        linkshell_id,
                        from_content_id,
                        from_name.clone(),
                        linkshell_name.clone(),
                    )
                    .await;
            }
            FromServer::SetCurrentMount(current_mount) => {
                connection.player_data.volatile.current_mount = current_mount as i32;
            }
            FromServer::FriendRemoved(their_content_id, their_name) => {
                connection
                    .friend_removed(their_content_id, their_name)
                    .await;
            }
            FromServer::NewLetterArrived => {
                connection.send_mailbox_status().await;
            }
            FromServer::PlayDirectorScene(content_handler_id, scene, flags) => {
                // A scene of the director's own, e.g. a quest battle's cutscene halfway through (see
                // ContentEventHandler, which tells the director when it's over.)
                connection
                    .start_event(
                        ObjectTypeId {
                            object_id: connection.player_data.character.actor_id,
                            object_type: ObjectTypeKind::None,
                        },
                        content_handler_id,
                        EventType::GameProgress,
                        1,
                        events,
                        lua_player,
                    )
                    .await;
                if events.last_mut().is_some() {
                    lua_player.play_scene(scene, SceneFlags::from_bits_retain(flags), Vec::new());
                }
            }
            FromServer::PlayDirectorCutscene(cutscene_id, content_handler_id, params) => {
                connection
                    .start_event(
                        ObjectTypeId {
                            object_id: connection.player_data.character.actor_id,
                            object_type: ObjectTypeKind::None,
                        },
                        content_handler_id,
                        EventType::GameProgress,
                        1,
                        events,
                        lua_player,
                    )
                    .await;
                if events.last_mut().is_some() {
                    lua_player.play_scene(
                        3,
                        SceneFlags::NO_DEFAULT_CAMERA
                            | SceneFlags::FADE_OUT
                            | SceneFlags::CONDITION_CUTSCENE
                            | SceneFlags::HIDE_HOTBAR
                            | SceneFlags::UNK1
                            | SceneFlags::DISABLE_STEALTH
                            | SceneFlags::INVIS_AOE,
                        std::iter::once(cutscene_id).chain(params).collect(),
                    );
                }
            }
            FromServer::FurniturePlaced(
                storage_id,
                slot,
                item_id,
                stain,
                position,
                indoors,
                rotation,
                plot_index,
            ) => {
                if indoors {
                    connection
                        .send_ipc_self(ServerZoneIpcSegment::new(
                            ServerZoneIpcData::InteriorFurniturePlaced {
                                storage_id,
                                slot,
                                item_id,
                                unk1: 1,
                                stain,
                                unk2: [0; 3],
                                rotation,
                                position,
                                unk3: [0; 4],
                            },
                        ))
                        .await;
                } else {
                    connection
                        .send_ipc_self(ServerZoneIpcSegment::new(
                            ServerZoneIpcData::ExteriorFurniturePlaced {
                                plot_index,
                                slot: slot as u8,
                                unk1: [0; 2],
                                item_id,
                                unk2: 0,
                                stain,
                                unk3: [0; 3],
                                rotation,
                                position,
                                unk4: 0,
                            },
                        ))
                        .await;
                }
            }
            FromServer::FurnitureTranslated(plot_info, slot, position, rotation, indoors) => {
                // TODO: This might need adjustment after double checking it in 7.5
                let plot = (plot_info.1 as u16) << 8;
                let index = slot & 0x00FF;
                connection
                    .send_ipc_self(ServerZoneIpcSegment::new(
                        ServerZoneIpcData::FurnitureTranslatedForObserver(
                            FurnitureTranslatedForObserver {
                                rotation,
                                plot_and_index: if indoors { slot } else { plot | index },
                                unk1: [0; 2],
                                position,
                                unk2: [0; 4],
                                ..Default::default()
                            },
                        ),
                    ))
                    .await;
            }
            FromServer::SyncMaxLevel(synced_level, synced_item_level) => {
                let current_level;
                {
                    let game_data = connection.game_data.lock();
                    current_level = connection.current_level(&game_data);
                }

                if current_level > synced_level as u16 {
                    connection.synced_level = Some(synced_level);
                    connection.synced_item_level = synced_item_level;
                    connection.update_class_info().await;
                    connection.send_stats().await;
                }
            }
            FromServer::EchoMultiplier(multiplier) => {
                connection.echo_multiplier = multiplier;
                connection.send_stats().await;
            }
            FromServer::MinimumItemLevel(item_level) => {
                connection.minimum_item_level = Some(item_level);
                connection.send_stats().await;
            }
            FromServer::PlayInitialCutscene(duration, settings, opening_flags) => {
                // Quest battles play the same opening scene, but without InstanceContent's parameters (as Sapphire does.)
                let is_quest_battle = connection
                    .content_handler_id
                    .is_some_and(|x| x.handler_type() == HandlerType::QuestBattle);
                // Air Force One's opening has flags and a fifth parameter of its own (`zone_connection/ride_shooting.rs`.)
                let ride_shooting = connection.ride_shooting_director().is_some();
                let params = if is_quest_battle {
                    Vec::new()
                } else {
                    vec![
                        0, // BGM, according to sapphire?
                        0,
                        0,
                        5,
                        if ride_shooting {
                            RIDE_SHOOTING_OPENING_VALUE
                        } else {
                            14400
                        },
                        0,
                        0,
                        0,
                        0,
                        0,
                        duration,
                        settings,
                    ]
                };
                // Duties' openings (0x0C53A401) are retail's; a quest battle's opening is a cutscene with its own flags
                // (`QUEST_BATTLE_OPENING_FLAGS` unless its script sets `OPENING_SCENE_FLAGS`): with the duties' ones
                // the UI stayed up and there was no skip button.
                let flags = if is_quest_battle {
                    SceneFlags::from_bits_retain(
                        opening_flags.unwrap_or(QUEST_BATTLE_OPENING_FLAGS),
                    )
                } else if ride_shooting {
                    SceneFlags::from_bits_retain(RIDE_SHOOTING_OPENING_FLAGS)
                } else {
                    SceneFlags::NO_DEFAULT_CAMERA
                        | SceneFlags::CONDITION_CUTSCENE
                        | SceneFlags::HIDE_HOTBAR
                        | SceneFlags::SILENT_ENTER_TERRI_ENV
                        | SceneFlags::SILENT_ENTER_TERRI_BGM
                        | SceneFlags::SILENT_ENTER_TERRI_SE
                        | SceneFlags::DISABLE_STEALTH
                        | SceneFlags::DISABLE_CANCEL_EMOTE
                        | SceneFlags::INVIS_AOE
                        | SceneFlags::UNK1
                };
                lua_player.play_scene(1, flags, params)
            }
            FromServer::FATEComplete(fate_id) => {
                connection.record_fate_completion();
                // A FATE ending unsyncs whoever synced to it (retail: the same packets as syncing, FateLevelSynced 0.)
                if connection.synced_fate == Some(fate_id) {
                    connection.set_fate_level_sync(fate_id, false).await;
                }
            }
            FromServer::PlayerDied => {
                connection.wear_gear_on_death().await;
            }
            FromServer::FateReward(fate_id) => {
                connection.give_fate_rewards(fate_id).await;
                connection.wear_gear_for_fate(fate_id).await;
            }
            FromServer::CheckTeleportSharingEligibility(aetheryte_id) => {
                connection.check_tele_sharing_eligibility(aetheryte_id);
            }
            FromServer::SetCharacterMode(mode, arg) => {
                connection.set_character_mode(mode, arg, false).await;
            }
            FromServer::ZoneDiceRoll(from_actor_id, roll_data) => {
                connection
                    .dice_roll_received(from_actor_id, roll_data)
                    .await;
            }
            _ => {
                tracing::error!(
                    "ZoneConnection {:#?} received a FromServer message we don't care about: {:#?}, ensure you're using the right client network or that you've implemented a handler for it if we actually care about it!",
                    client_handle.id,
                    msg
                );
            }
        }
    }

    // Run whatever the message had a script queue (the scene after an event action's progress bar, a kill's
    // variables, a won quest battle's scenes...) now: left queued, it waits for the client's next packet, which can be
    // several seconds away when the player is standing still (seen: the gangplank's yes/no 2.7 and 6 s late.)
    if !lua_player.queued_tasks.is_empty()
        || !connection.queued_tasks.is_empty()
        || !lua_player.zone_data.queued_segments.is_empty()
    {
        lua_player.queued_tasks.append(&mut connection.queued_tasks);
        if connection.process_lua_player(lua_player, events).await {
            connection.process_lua_player(lua_player, events).await;
        }
        lua_player.player_data = connection.player_data.clone();
    }
}

/// How long until a moment given in seconds since the epoch (a little after it, so the second has begun.)
fn until_unix_time(at: u64) -> std::time::Duration {
    (SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(at))
        .duration_since(SystemTime::now())
        .unwrap_or_default()
        + std::time::Duration::from_millis(50)
}

async fn client_loop(
    mut connection: ZoneConnection,
    mut internal_recv: UnboundedReceiver<FromServer>,
    client_handle: ClientHandle,
) {
    let mut lua_player = LuaPlayer::default();

    let mut buf = vec![0; RECEIVE_BUFFER_SIZE];
    let mut client_handle = client_handle.clone();
    client_handle.actor_id = connection.player_data.character.actor_id;

    // Do an initial update otherwise it may be uninitialized for the first packet that needs Lua
    lua_player.player_data = connection.player_data.clone();

    // tell the server we exist, now that we confirmed we are a legitimate connection
    connection
        .handle
        .send(ToServer::NewClient(client_handle.clone()))
        .await;

    // This is living outside of ZoneConnection (which is weird)
    // because we need the functions in EventHandler to mutate it.
    // Of course, Rust's mutability rules disallow that.
    let mut events: Vec<(Box<dyn EventHandler>, Event)> = Vec::new();

    let mut autosave = tokio::time::interval(AUTOSAVE_INTERVAL);
    autosave.tick().await; // the first tick is immediate, there's nothing new to save yet

    loop {
        let gate_wait = until_unix_time(connection.next_gate_change());
        tokio::select! {
            biased; // client data should always be prioritized
            n = connection.socket.read(&mut buf) => {
                match n {
                    Ok(n) => {
                        if !process_packet(&mut connection, &mut lua_player, &mut events, client_handle.clone(), n, &buf).await {
                            break;
                        }
                    },
                    Err(_) => {
                        tracing::info!("ZoneConnection {:#?} was killed because of a network error!", client_handle.id);
                        break;
                    },
                }
            }
            msg = internal_recv.recv() => process_server_msg(&mut connection, &mut lua_player, &mut events, client_handle.clone(), msg).await,
            _ = autosave.tick() => {
                if connection.player_data.character.actor_id.is_valid() && !connection.gracefully_logged_out {
                    connection.autosave();
                }
            }
            // A GATE beginning or ending (`gate.rs`), told to whoever is in the Gold Saucer.
            _ = tokio::time::sleep(gate_wait) => {
                connection.gate_changed().await;
            }
            // A connection that dropped without closing (the network went away, the PC lost power) never reads 0 bytes,
            // so it would wait here forever without logging the player out and saving them. The client talks to us
            // every few seconds while it's connected, even on loading screens.
            _ = tokio::time::sleep_until((connection.last_keep_alive + DEAD_CONNECTION_TIMEOUT).into()) => {
                tracing::info!("Dropping zone connection from {:?} because it went silent", client_handle.id);
                break;
            }
        }
    }

    // forcefully log out the player if they weren't logging out but force D/C'd
    if connection.player_data.character.actor_id.is_valid() {
        if !connection.gracefully_logged_out {
            tracing::info!(
                "Forcefully logging out connection {:#?}...",
                client_handle.id
            );
            connection.begin_log_out().await;
        }
        connection
            .handle
            .send(ToServer::Disconnected(
                connection.id,
                connection.player_data.character.actor_id,
            ))
            .await;

        if connection.is_in_party() {
            connection
                .handle
                .send(ToServer::PartyMemberOffline(
                    connection.party_id,
                    connection.player_data.character.service_account_id as u64,
                    connection.player_data.character.content_id as u64,
                    connection.player_data.character.actor_id,
                    connection.player_data.character.name.clone(),
                ))
                .await;
        }
    }
}

async fn root() -> String {
    "1".to_string()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = get_config();

    let addr = config.world.get_socketaddr();

    let listener = TcpListener::bind(addr).await.unwrap();

    let database = Arc::new(Mutex::new(WorldDatabase::new()));
    let lua = Arc::new(Mutex::new(KawariLua::new()));
    let game_data = Arc::new(Mutex::new(GameData::new()));

    tracing::info!("Server started on {addr}");
    kawari_world::auto_opcodes::adapt_to_game(&config.filesystem.game_path());
    kawari_world::fork_opcodes::scan_executable(&config.filesystem.game_path());
    kawari_world::fork_opcodes::log_outstanding();

    {
        let mut lua = lua.lock();
        if let Err(err) = lua.init(game_data.clone()) {
            tracing::warn!("Failed to load Init.lua: {:?}", err);
        }
    }

    {
        let mut database = database.lock();
        database.do_cleanup_tasks();
    }

    let (handle, _) = spawn_main_loop(game_data.clone(), database.clone());

    // This is a static healthcheck meant for the Kawari Toolbox plugin.
    let app = Router::new().route("/healthcheck", get(root));

    let mut healthcheck_addr = addr;
    healthcheck_addr.set_port(config.world.healthcheck_port);
    let healthcheck_listener = tokio::net::TcpListener::bind(healthcheck_addr)
        .await
        .unwrap();

    tokio::spawn(async move {
        axum::serve(healthcheck_listener, app).await.unwrap();
    });

    loop {
        if let Ok((socket, addr)) = listener.accept().await {
            let id = handle.next_id();
            tracing::info!("New connection from {addr}, now referring to it as {id:?}");

            spawn_initial_setup(
                id,
                socket,
                lua.clone(),
                database.clone(),
                game_data.clone(),
                handle.clone(),
            );
        }
    }
}

/// The maps with up to 16 discovered parts, from a character's saved map discovery (see `MAP_DISCOVERY_BITMASK_SIZE`.)
fn map_discovery_u16(data: &[u8]) -> Vec<u16> {
    (0..DISCOVERY_MAPS_WITH_UP_TO_16_REGIONS_BITMASK_SIZE)
        .map(|i| {
            let byte = |at: usize| data.get(at).copied().unwrap_or_default();
            u16::from_le_bytes([byte(i * 2), byte(i * 2 + 1)])
        })
        .collect()
}

/// The maps with up to 32 discovered parts, from a character's saved map discovery.
fn map_discovery_u32(data: &[u8]) -> Vec<u32> {
    let start = DISCOVERY_MAPS_WITH_UP_TO_16_REGIONS_BITMASK_SIZE * 2;
    (0..DISCOVERY_MAPS_WITH_UP_TO_32_REGIONS_BITMASK_SIZE)
        .map(|i| {
            let byte = |at: usize| data.get(start + at).copied().unwrap_or_default();
            u32::from_le_bytes([
                byte(i * 4),
                byte(i * 4 + 1),
                byte(i * 4 + 2),
                byte(i * 4 + 3),
            ])
        })
        .collect()
}
