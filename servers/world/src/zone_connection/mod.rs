use std::{
    collections::HashMap,
    sync::Arc,
    time::{Instant, SystemTime},
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use crate::{
    Content, GameData, Recipe, Unlock,
    database::{
        AetherCurrent, Aetheryte, Buddy, Character, ClassJob, Companion, Customize, Friends,
        GrandCompany, Mentor, Quest, SearchInfo, Volatile,
    },
    lua::{KawariLua, LuaTask},
};
use kawari::{
    common::{HandlerId, ObjectId, Position, WarpType, timestamp_secs},
    config::WorldConfig,
    ipc::zone::{
        ActorControlCategory, ApartmentList, ApartmentListEntry, CWLSMemberListEntry,
        ClientTriggerCommand, ClientZoneIpcSegment, Condition, Conditions, DutyFinderSetting,
        DyeInformation, GrandCompany as IpcGrandCompany, LetterPreview, PlayerEntry,
        ServerZoneIpcData, ServerZoneIpcSegment,
    },
    opcodes::ServerZoneIpcType,
    packet::{
        ConnectionState, ConnectionType, IpcSegmentHeader, PacketSegment, SegmentData, SegmentType,
        ServerIpcSegmentHeader, parse_packet, send_keep_alive, send_packet,
    },
};

use super::{
    WorldDatabase,
    common::{ClientId, ServerHandle},
    inventory::{BuyBackList, HousingInventory, Inventory},
};

mod achievement;
mod actor;
mod chat;
mod chocobo_taxi;
mod companion;
mod effect;
mod event;
mod friends;
mod gate;
mod gathering;
mod glamour;
mod hunting_log;
pub use achievement::{
    COUNTER_GATES_COMPLETED, COUNTER_GOLD_SAUCER_MGP, COUNTER_JUMBO_CACTPOT_MGP,
    COUNTER_MINI_CACTPOT_MGP,
};
pub use gathering::GatheringState;
mod item;
mod linkshell;
mod lua;
mod mail;
mod marketboard;
mod mender;
mod party;
mod player_currency;
pub(crate) mod quest;
pub use player_currency::{ACHIEVEMENT_CERTIFICATES_TOTAL, PLAYER_CURRENCIES, certificates_owed};
pub use quest::RETAINER_UNLOCK_LINK;
mod retainer;
mod ride_shooting;
pub use ride_shooting::{
    OPENING_SCENE_FLAGS as RIDE_SHOOTING_OPENING_FLAGS,
    OPENING_SCENE_VALUE as RIDE_SHOOTING_OPENING_VALUE,
};
mod social;
mod spiritbond;
pub use spiritbond::{GearUse, MaterializeRequest};
mod stats;
pub use stats::{BaseParameters, TENACITY_SCALE};
mod unlock;
mod zone;

#[derive(Debug, Default, Clone)]
pub struct TeleportQuery {
    pub aetheryte_id: u16,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum TeleportReason {
    #[default]
    NotSpecified,
    /// Teleporting/Returning to an Aetheryte or shared
    Aetheryte,
}

/// Quest information stored in the database.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PersistentQuest {
    /// ID of the quest.
    pub id: u16,
    /// Sequence in the quest.
    pub sequence: u8,
    /// The quest's variables (UI8A through UI8F), which the quest script and the client track objectives with.
    #[serde(default)]
    pub variables: [u8; 6],
    /// The class/job it was accepted as: retail's active quest list carries it (every quest in Bob's captures), and
    /// the client's quest scripts can branch on it; Kawari sent 0 (`questkit fields`, 2026-10-26).
    #[serde(default)]
    pub accept_class_job: u8,
}

/// Persistent player data.
/// Please try to keep fields in here to a minimum, specifically stuff that may be persistent and/or accessed from Lua.
#[derive(Debug, Default, Clone)]
pub struct PlayerData {
    pub character: Character,
    pub classjob: ClassJob,
    pub volatile: Volatile,
    pub inventory: Inventory,
    pub city_state: u8,
    pub mentor: Mentor,
    pub search_info: SearchInfo,

    pub teleport_query: TeleportQuery,
    pub gm_invisible: bool,

    pub item_sequence: u32,
    pub shop_sequence: u32,
    /// The server-side copy of NPC shop buyback lists.
    pub buyback_list: BuyBackList,
    pub unlock: Unlock,
    pub content: Content,
    pub aetheryte: Aetheryte,
    pub aether_current: AetherCurrent,
    pub companion: Companion,
    pub quest: Quest,
    pub saw_inn_wakeup: bool,
    pub friends: Friends,
    pub grand_company: GrandCompany,
    // TODO: These inventories need to be made persistent and also vary per property, this is just for initial support
    pub house_inventory: HousingInventory,
    pub buddy: Buddy,
    pub equipped_glasses_ids: [u16; 2],
    pub customize: Customize,
}

/// Various obsfucation-related bits like the seeds and keys for this connection.
#[derive(Debug, Default, Clone)]
pub struct ObsfucationData {
    pub seed1: u8,
    pub seed2: u8,
    pub seed3: u32,
}

/// Represents a single connection between an instance of the client and the zone portion of the world server.
pub struct ZoneConnection {
    pub config: WorldConfig,
    pub socket: TcpStream,

    pub state: ConnectionState,
    pub player_data: PlayerData,

    pub id: ClientId,
    pub handle: ServerHandle,

    pub database: Arc<Mutex<WorldDatabase>>,
    pub lua: Arc<Mutex<KawariLua>>,
    pub game_data: Arc<Mutex<GameData>>,

    pub teleport_reason: TeleportReason,
    pub active_minion: u32,
    /// The player's party id number, used for networking party-related events
    pub party_id: u64,
    pub is_party_leader: bool,
    /// The player's status when connecting/reconnecting. If true, they need to rejoin their party.
    pub rejoining_party: bool,
    /// The player's currently active quests.
    pub login_time: Option<SystemTime>,
    /// The character's total playtime (minutes) when they logged in. The database copy moves on with every autosave,
    /// so the current total is this plus the session so far, never the database's plus the session.
    pub playtime_at_login: i64,
    pub glamour_information: Option<ClientTriggerCommand>,

    pub last_keep_alive: Instant,

    /// Whether the player was gracefully logged out
    pub gracefully_logged_out: bool,

    pub obsfucation_data: ObsfucationData,

    // TODO: support more than one content in the queue
    pub queued_content: Option<u16>,
    /// The Duty Support members the player chose for `queued_content` (`RegisterDutySupport`), sent with JoinContent.
    pub duty_support: Option<Vec<u8>>,
    pub duty_settings: Option<DutyFinderSetting>,
    /// The Duty Finder settings the duty the player is in was entered with (`join_content`), until they leave it for
    /// a zone that isn't one: what the duty gives depends on them (`server/duty_finder.rs`).
    pub entered_with: Option<DutyFinderSetting>,
    /// The roulette queued for (`QueueRoulette`), and the one the duty they're in was rolled by, for its first clear of
    /// the day's bonus (`roulette_cleared`.)
    pub queued_roulette: Option<u8>,
    pub entered_roulette: Option<u8>,
    /// The levels of the bots in their party (its last list, `send_bot_party_update`), for Limited Leveling Roulette.
    pub bot_party_levels: Vec<u8>,
    /// The party bonus in percent from the last party list sent (`stats::party_bonus_percent`.)
    pub party_bonus: u8,
    /// Well Fed's param while the player has it (`FromServer::WellFed`), for food's stats.
    pub well_fed: Option<u16>,

    pub conditions: Conditions,

    /// List of queued tasks from the server.
    pub queued_tasks: Vec<LuaTask>,

    /// Information from before we entered the content.
    pub old_zone_id: u16,
    pub old_position: Position,
    pub old_rotation: f32,

    /// Information about the current content.
    pub event_handler_id: Option<HandlerId>, // TODO: replace with a helper function
    /// The base ID the next event's script gets as `BASE_ID` when it isn't an event NPC or object of the zone's (a
    /// key item used on a battle NPC: `FromServer::EventItemUsed`); taken by `start_event`.
    pub event_base_override: Option<u32>,
    pub recipe: Option<Recipe>,
    /// The synthesis under way (`events/crafting.rs`.)
    pub craft: Option<Box<crate::events::CraftSession>>,
    /// How many a quick synthesis being started makes, until its materials are chosen.
    pub quick_synthesis: Option<u32>,
    /// The materia extraction or self-repair under way (`zone_connection/spiritbond.rs`.)
    pub materialize: Option<MaterializeRequest>,
    /// The gathering points as this player sees them (`zone_connection/gathering.rs`.)
    pub gathering: GatheringState,
    /// What the client has been told each inventory slot holds (every UpdateItem and UpdateInventorySlot sent, its
    /// sequence left out), and whether the whole inventory has gone once (`send_inventory`): after that a resend goes
    /// slot by slot, only what changed.
    pub shown_items: std::collections::HashMap<
        (kawari::common::ContainerType, u16),
        kawari::ipc::zone::ItemInfo,
    >,
    pub inventory_shown: bool,
    /// A gearset was equipped and its Equip hasn't come back yet: retail's ActorControlSelf 805 follows it
    /// (`update_equip`.)
    pub gearset_equipped: bool,
    pub synced_level: Option<u8>,
    /// The item level the content syncs to, if it has its own (see `GameData::find_content_synced_item_level`.)
    pub synced_item_level: Option<u16>,
    /// Minimum IL in the duty they're in: their gear capped at this item level, synced or not
    /// (`FromServer::MinimumItemLevel`.)
    pub minimum_item_level: Option<u16>,
    /// The Echo's (and Epic Echo's) multiplier on maximum HP in the duty they're in (`FromServer::EchoMultiplier`.)
    pub echo_multiplier: f32,
    /// The FATE the player synced to, which unsyncs them when it ends.
    pub synced_fate: Option<u32>,
    /// Player Search results.
    pub search_results: Vec<PlayerEntry>,
    /// The Player Search's current sequence value. Increases by 10 every time the client requests more results.
    pub search_index: usize,
    /// Friend list results.
    pub friend_results: Vec<PlayerEntry>,
    /// The friend list's current sequence value. Increases by 10 every time the client requests more results.
    pub friend_index: usize,
    /// CWLS member results sent when the player opens the CWLS menu or picks a different linkshell in the same menu.
    pub cwls_results: Vec<CWLSMemberListEntry>,
    /// CWLS member index. Increases by 8 every time the client requests more results.
    pub cwls_index: usize,
    /// The player's Moogle Delivery Service previews. Used to populate the Moogle Delivery Service window when interacting with a Delivery Moogle or a Letter Box.
    pub mail_results: Vec<LetterPreview>,
    /// The current index into the mailbox previews. Increases by 5 every time the clint requests more previews.
    pub mail_index: usize,
    /// Whether the player is spawned in or not.
    pub spawned_in: bool,
    /// The last teleport offered to this player. Only one can be kept at a time.
    pub offered_teleport: Option<TeleportQuery>,
    /// Whether the player is trading with another player or not.
    pub is_trading: bool,
    /// Current dye action information.
    pub dyeing_information: Option<DyeInformation>,
    /// Temporary data for marketboard requests.
    pub marketboard_request_item_id: u32,
    /// Temporary data for the spectator debug command.
    pub hide_spectator_ui: bool,
    /// Whether the player is spawned in before.
    pub initial_login: bool,
    /// Mapping of motivation NPCs to their FATE ids for easier lookup. Used in the FATE event handler.
    pub fate_motivation_npcs: HashMap<ObjectId, u16>,
    /// Used only for the local player's PlayerSpawn.
    pub content_handler_id: Option<HandlerId>,
    /// The duty (ContentFinderCondition) the player is in, 0 out in the world.
    pub content_finder_condition_id: u16,
    /// The InstanceContent the player cleared and hasn't left yet: quests hear of it as they leave (`onDutyCleared`).
    pub duty_cleared: Option<u16>,
    /// The event action currently being cast by a quest, see `LuaTask::EventAction`.
    pub event_action_id: u32,
    /// A nested event whose packets a script started (`start_nested_event`) and hasn't finished yet.
    pub nested_event: Option<HandlerId>,
    /// Whether to stay in a won quest battle after "Duty Complete", until its director sends us back.
    pub quest_battle_stay: bool,
    /// The scene a won quest battle plays before sending us back (`QUEST_BATTLE_SCENE_FINISH` unless its director
    /// picks another), or `None` while it hasn't been won.
    pub quest_battle_finish_scene: Option<u16>,
    /// A LogMessage row the won battle's quest shows (from its own event) as the finish scene ends: the quest and the
    /// row (`complete_quest_battle`'s `quest_log`.)
    pub quest_battle_finish_log: Option<(u32, u32)>,
    /// The scene flags `quest_battle_finish_scene` is played with (`QUEST_BATTLE_SCENE_FINISH_FLAGS` unless its
    /// director gives others.)
    pub quest_battle_finish_flags: u32,
    /// Whether the player can offer this teleport to their party.
    pub can_share_teleport: bool, // TODO: remove this and add more teleport reasons
    /// The current EXP chain: how many kills in a row, and when the last one was. See `on_battle_npc_killed`.
    pub exp_chain: (u32, Option<std::time::Instant>),
    /// The retainer called at a summoning bell, if any (their items are in `player_data.inventory.retainer`.)
    pub called_retainer: Option<retainer::CalledRetainer>,
    /// The appearance chosen at the retainer vocate (voice, CustomizeData), until the retainer is named.
    pub pending_retainer_appearance: Option<(u8, physis::savedata::chardat::CustomizeData)>,
    /// The text the client typed into the current event (a retainer's name), see `kawari::ipc::zone::event_text`.
    pub pending_event_text: Option<String>,
    /// The Scenario Guide last sent (the quest completed last, the next ones and their issuer locations), so it's
    /// sent only when it changes, as retail did.
    pub scenario_guide_sent: Option<(u32, Vec<(u32, u32)>)>,
    /// The Gold Saucer game being played (a Mini Cactpot ticket, an arcade machine's play), while its event lasts.
    pub gold_saucer_game: Option<crate::events::GoldSaucerGame>,
    /// The GATE (its slot, `gate.rs`) the player joined: each can be played once.
    pub gate_joined: Option<u64>,
    /// When the GATEs were last looked at (`gate_changed`), so each beginning and end is told once.
    pub gate_checked: u64,
    /// Whether this Air Force One's score has been taken (the client sends it again and again until it leaves.)
    pub ride_shooting_scored: bool,
    /// The layer filter key the client was last given for the zone it's in (ZoneInit, ActorSetPos.)
    pub layer_filter_shown: u32,
    /// When to see whether the story moved the zone's layer filter on (after an event; `refresh_layer_filter`.)
    pub layer_filter_check: Option<std::time::Instant>,
    /// Whether the player is watching a cutscene (a scene with `SceneFlags::CONDITION_CUTSCENE`): retail's condition
    /// OccupiedInCutSceneEvent, Viewing Cutscene among their online statuses, and their party told (ActorControl 519;
    /// `set_watching_cutscene`.)
    pub watching_cutscene: bool,
    /// The last of retail's count of the times the server put the player somewhere (`position_sequence_after`.)
    pub position_sequence: u32,
    /// The chocobo porter's ride being warped to (ChocoboTaxi row, its Location), until the client has finished zoning.
    pub chocobo_taxi: Option<(u32, u32)>,
    /// The chocobo porter's ride under way (ChocoboTaxi row), until the client says it has arrived.
    pub riding_chocobo_taxi: Option<u32>,
    /// Packet capture for debugging, if enabled in the config.
    pub capture: crate::capture::PacketCapture,
}

impl ZoneConnection {
    pub fn parse_packet(&mut self, data: &[u8]) -> Vec<PacketSegment<ClientZoneIpcSegment>> {
        let segments = parse_packet(data, &mut self.state);
        for segment in &segments {
            self.capture.record_incoming(segment);
        }
        segments
    }

    /// Sends an IPC segment to the player, where the source actor is also the player.
    /// The layer filter the client is shown `territory` with, for this character's quest progress (`layer_filters`.)
    pub fn layer_filter_key(&self, territory: u16) -> u32 {
        let quests = &self.player_data.quest;
        crate::layer_filters::filter_key(territory, |quest, sequence, completed| {
            quests.completed.contains(quest)
                || !completed
                    && quests.active.0.iter().any(|active| {
                        active.id as u32 == quest
                            && sequence.is_none_or(|needed| active.sequence >= needed)
                    })
        })
    }

    /// After an event: if the story has moved the zone's layer filter on (Back from the Wood's cutscene in the Waking
    /// Sands), the player is warped where they stand so the client loads the new one, as retail did: PrepareZoning to
    /// the same territory with warp type 18, then ActorSetPos (warp type 18, arg 2) with the new key, 1.5 s later
    /// (Bob's Chronofoil captures cc62ad94, 9df4ff01, 3a705e18: every time the key changed within a visit.) Without it
    /// the client kept the layers it came in with until the next zone-in.
    pub async fn refresh_layer_filter(&mut self) {
        self.layer_filter_check = None;
        let territory = self.player_data.volatile.zone_id as u16;
        if !self.spawned_in || !crate::layer_filters::has_filters(territory) {
            return;
        }
        let key = self.layer_filter_key(territory);
        if key == self.layer_filter_shown {
            return;
        }
        tracing::info!(
            "Layer filter of {territory} now {key} (shown {}): warped where they stand",
            self.layer_filter_shown
        );
        let position = self.player_data.volatile.position;
        let rotation = self.player_data.volatile.rotation as f32;
        self.change_zone(
            territory,
            Some(position),
            Some(rotation),
            WarpType::Unk18,
            0,
        )
        .await;
    }

    pub async fn send_ipc_self(&mut self, ipc: ServerZoneIpcSegment) {
        // What the client now believes each slot holds (`shown_items`.)
        if let ServerZoneIpcData::UpdateItem(info) | ServerZoneIpcData::UpdateInventorySlot(info) =
            &ipc.data
        {
            let key = (info.container, info.slot);
            if info.item_id == 0 || info.quantity == 0 {
                self.shown_items.remove(&key);
            } else {
                self.shown_items.insert(
                    key,
                    kawari::ipc::zone::ItemInfo {
                        sequence: 0,
                        ..info.clone()
                    },
                );
            }
        }
        // This is meant to protect against stack-smashing in nested futures
        Box::pin(self.send_ipc_from(self.player_data.character.actor_id, ipc)).await;
    }

    /// Sends an IPC segment to the player, where the source actor can be specified.
    pub async fn send_ipc_from(&mut self, source_actor: ObjectId, mut ipc: ServerZoneIpcSegment) {
        // Moved within a zone the story changes (a quest's cutscene ending in the Waking Sands): the layer filter for
        // where the story is now, as retail's ActorSetPos carried (`layer_filters`; 0 showed every stage at once.)
        if let ServerZoneIpcData::ActorSetPos(set_pos) = &mut ipc.data
            && (source_actor == ObjectId::default()
                || source_actor == self.player_data.character.actor_id)
        {
            if set_pos.transition_territory_filter_key == 0 {
                set_pos.transition_territory_filter_key =
                    self.layer_filter_key(self.player_data.volatile.zone_id as u16);
            }
            self.layer_filter_shown = set_pos.transition_territory_filter_key;
        }
        // Leaving the Gold Saucer while a GATE is on ends its director, after the PrepareZoning (`leave_gate`.)
        let leaving_gold_saucer =
            crate::gate::leaves_gold_saucer(&ipc.data, self.player_data.volatile.zone_id as u16);
        // Putting the player somewhere is counted for the client after it (`position_sequence_after`.)
        let puts_the_player =
            puts_the_player(&ipc.data, source_actor, self.player_data.character.actor_id);
        // What the client shows the zone with now (`refresh_layer_filter`), and a zone change under way brings its own.
        match &ipc.data {
            ServerZoneIpcData::ZoneInit(init) => {
                self.layer_filter_shown = init.transition_territory_filter_key;
                self.position_sequence = 0;
            }
            ServerZoneIpcData::PrepareZoning(_) => self.layer_filter_check = None,
            _ => {}
        }
        let segment = PacketSegment {
            source_actor: if source_actor == ObjectId::default() {
                self.player_data.character.actor_id
            } else {
                source_actor
            },
            target_actor: self.player_data.character.actor_id,
            segment_type: SegmentType::Ipc,
            data: SegmentData::Ipc(ipc),
        };

        // Ditto from above
        Box::pin(self.send_segment(segment)).await;
        if leaving_gold_saucer {
            Box::pin(self.leave_gate()).await;
        }
        if puts_the_player {
            self.position_sequence = position_sequence_after(self.position_sequence);
            let category = ActorControlCategory::MovementRelatedUnk {
                unk1: self.position_sequence,
            };
            Box::pin(self.actor_control_self(category)).await;
        }
    }

    pub async fn send_segment(&mut self, segment: PacketSegment<ServerZoneIpcSegment>) {
        self.capture.record_outgoing(&segment);

        // Ditto as above
        Box::pin(send_packet(
            &mut self.socket,
            &mut self.state,
            ConnectionType::Zone,
            self.config.compression(),
            &[segment],
        ))
        .await;
    }

    pub async fn initialize(&mut self) {
        self.player_data.item_sequence = 0;
        self.player_data.shop_sequence = 0;

        // We have send THEM a keep alive
        {
            self.send_segment(PacketSegment {
                segment_type: SegmentType::KeepAliveRequest,
                data: SegmentData::KeepAliveRequest {
                    id: 0xE0037603u32,
                    timestamp: timestamp_secs(),
                },
                ..Default::default()
            })
            .await;
        }

        self.send_segment(PacketSegment {
            segment_type: SegmentType::Initialize,
            data: SegmentData::Initialize {
                actor_id: self.player_data.character.actor_id,
                timestamp: timestamp_secs(),
            },
            ..Default::default()
        })
        .await;
    }

    /// The character's total playtime in minutes, including this session so far.
    pub fn playtime_minutes(&self) -> i64 {
        let session = match self
            .login_time
            .map(|login_time| SystemTime::now().duration_since(login_time))
        {
            Some(Ok(session_length)) => (session_length.as_secs() / 60) as i64,
            Some(Err(e)) => {
                tracing::error!(
                    "Unable to determine the current session's playtime, due to an error: {e}"
                );
                0
            }
            None => 0,
        };
        self.playtime_at_login + session
    }

    /// Writes the player's progress back to the database while they're still playing, so a dropped connection or a
    /// server that goes down only loses the last little while. Saved like logging out would (playtime included),
    /// except they stay online.
    pub fn autosave(&mut self) {
        let mut data = self.player_data.clone();
        // Playtime is saved as it goes too, so anything counting it (Kawari Web's play coins) keeps up.
        if self.login_time.is_some() {
            data.character.time_played_minutes = self.playtime_minutes();
        }

        // In an instance, save where they entered it from, as logging out does, so they don't come back to an
        // instance that's gone.
        if self.conditions.has_condition(Condition::BoundByDuty) {
            data.volatile.zone_id = self.old_zone_id as i32;
            data.volatile.position = self.old_position;
            data.volatile.rotation = self.old_rotation as f64;
        }

        let mut database = self.database.lock();
        database.commit_player_data(&data);
    }

    pub async fn begin_log_out(&mut self) {
        // A retainer still out at a bell goes home (saving what they hold.)
        self.release_called_retainer().await;

        // Mark the player as offline in the db.
        self.player_data.volatile.is_online = false;

        // If we were last in an instance, tell the server we're outside of it so we don't get stuck/crash.
        if self.conditions.has_condition(Condition::BoundByDuty) {
            self.player_data.volatile.zone_id = self.old_zone_id as i32;
            self.player_data.volatile.position = self.old_position;
            self.player_data.volatile.rotation = self.old_rotation as f64;
        }

        // Update playtime
        if self.login_time.is_some() {
            self.player_data.character.time_played_minutes = self.playtime_minutes();
        }

        // Write the player back to the database
        {
            let mut database = self.database.lock();
            database.commit_player_data(&self.player_data);
        }

        // Don't bother sending these if the client forcefully D/C'd.
        if self.gracefully_logged_out {
            // Set the client's conditions for logout preparation
            self.conditions.set_condition(Condition::LoggingOut);
            self.send_conditions().await;

            // Tell the client we're ready to disconnect at any moment
            {
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LogOutComplete {
                    unk: [1, 0, 0, 0, 0, 0, 0, 0],
                });
                self.send_ipc_self(ipc).await;
            }
        }
    }

    pub async fn send_arbitrary_packet(&mut self, op_code: u16, data: Vec<u8>) {
        let ipc = ServerZoneIpcSegment {
            header: ServerIpcSegmentHeader::from_opcode(ServerZoneIpcType::Unknown(op_code)),
            data: ServerZoneIpcData::Unknown { unk: data },
        };
        self.send_ipc_self(ipc).await;
    }

    pub async fn send_keep_alive(&mut self, id: u32, timestamp: u32) {
        send_keep_alive::<ServerZoneIpcSegment>(
            &mut self.socket,
            &mut self.state,
            ConnectionType::Zone,
            id,
            timestamp,
        )
        .await;
    }

    /// In a duty entered as an unrestricted party without Level Sync (`duty_finder::unsynced_unrestricted`): no EXP,
    /// spiritbond or item drops from enemies.
    pub fn unsynced_unrestricted(&self) -> bool {
        self.content_finder_condition_id != 0
            && self
                .entered_with
                .is_some_and(crate::server::duty_finder::unsynced_unrestricted)
    }

    pub async fn register_for_content(
        &mut self,
        content_ids: [u16; 5],
        settings: DutyFinderSetting,
    ) {
        self.queued_content = Some(content_ids[0]);
        self.duty_settings = Some(settings);
        // A roulette's handler sets it again after this.
        self.queued_roulette = None;
        // Through the Duty Finder unless `RegisterDutySupport` says otherwise after this.
        self.duty_support = None;

        // update
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ContentFinderUpdate {
                state1: 1,
                language_flags: 0,
                classjob_id: self.player_data.classjob.current_class as u8, // TODO: store what they registered with, because it can change
                roulette_id: 0,
                began_queue: 0,
                duty_settings: settings,
                content_ids: content_ids.map(u32::from),
            });
            self.send_ipc_self(ipc).await;
        }

        // found
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ContentFinderFound {
                queue_state: 3,
                unk1: [0; 7],
                duty_settings: settings,
                unk2: [0; 12],
                content_id: content_ids[0],
                unk3: [0; 10],
            });
            self.send_ipc_self(ipc).await;
        }
    }

    pub async fn send_playtime(&mut self) {
        if let Some(login_time) = self.login_time {
            // Not the database's total: autosaves keep that up to date with this session already.
            let time_played_minutes = self.playtime_at_login;

            // In case something goes wrong with calculating the current session's playtime, we'll send the old total by default.
            let mut total_play_time = time_played_minutes;
            match SystemTime::now().duration_since(login_time) {
                Ok(session_length) => {
                    total_play_time = (session_length.as_secs() / 60) as i64 + time_played_minutes;

                    // Retail doesn't do this, but it's a nice QoL thing to have.
                    self.send_notice(
                        &format!(
                            "Total Play Time this Session: {} hours, {} minutes, {} seconds",
                            session_length.as_secs() / 3600,
                            (session_length.as_secs() / 60) % 60,
                            session_length.as_secs() % 60
                        )
                        .to_string(),
                    )
                    .await;
                }
                Err(e) => {
                    tracing::error!(
                        "Unable to determine the current session's playtime, due to an error: {e}",
                    );
                }
            }

            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::Playtime {
                duration: total_play_time as u32,
            });

            self.send_ipc_self(ipc).await;
        }
    }

    // TODO: break this out into its own housing file eventually
    pub async fn send_apartment_list(&mut self, starting_index: u32) {
        let mut apartments = vec![ApartmentListEntry::default(); ApartmentListEntry::COUNT];
        apartments[0] = ApartmentListEntry {
            resident_online_status_mask: self.get_online_status_mask(),
            resident_zone_id: self.player_data.volatile.zone_id as u16,
            resident_name: self.player_data.character.name.clone(),
            apartment_description: "A test apartment provided to you by Kawari!".to_string(),
            ..Default::default()
        };

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ApartmentList(ApartmentList {
            content_id: self.player_data.character.content_id as u64,
            flags: 128,
            ward_id: 0,
            zone_id: self.player_data.volatile.zone_id as u16, // TODO: Apartment lists can be requested from apartments themselves, so this wouldn't make sense there!
            world_id: self.config.world_id,
            list_index: starting_index,
            apartments,
        }));
        self.send_ipc_self(ipc).await;
    }

    pub async fn send_grand_company_info(&mut self) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::GrandCompanyInfo {
            active_company_id: self.player_data.grand_company.active_company,
            maelstrom_rank: self.grand_company_rank(IpcGrandCompany::Maelstrom).unwrap(),
            twin_adder_rank: self.grand_company_rank(IpcGrandCompany::Adders).unwrap(),
            immortal_flames_rank: self.grand_company_rank(IpcGrandCompany::Flames).unwrap(),
        });
        self.send_ipc_self(ipc).await;
    }

    pub fn grand_company_rank(&self, company: IpcGrandCompany) -> Option<u8> {
        if company != IpcGrandCompany::None {
            let company_index = company as usize - 1;
            return Some(self.player_data.grand_company.company_ranks.0[company_index]);
        }

        None
    }

    /// Promotes the player in their Grand Company, as the personnel officer does once the client's checks pass (its
    /// script's scene 2: the rank-up quest, the Hunting Log and the seals): the seals the rank asks for are paid
    /// (GrandCompanyRank's RequiredSeals of the rank held: 2,000 from Private Third Class), then the rank, then
    /// GrandCompanyInfo. Retail's packets for it aren't recorded (Bob's captures never reached a promotion.) Returns
    /// false if there's no company, no next rank, or not enough seals.
    pub async fn promote_grand_company(&mut self) -> bool {
        let company = self.player_data.grand_company.active_company;
        let Some(rank) = self.grand_company_rank(company) else {
            return false;
        };
        let cost = self.game_data.lock().get_gc_rank_seals(rank);
        let next = self.game_data.lock().get_gc_rank_seals(rank + 1);
        let (Some((_, required)), Some(_)) = (cost, next) else {
            tracing::warn!("Grand Company {company:?}: no rank after {rank}");
            return false;
        };
        let slot = company as u16;
        let seals = &mut crate::inventory::Storage::get_slot_mut(
            &mut self.player_data.inventory.currency,
            slot,
        )
        .quantity;
        if (*seals as u32) < required {
            tracing::warn!(
                "Grand Company {company:?}: {required} seals needed to leave rank {rank}"
            );
            return false;
        }
        *seals -= required as i32;
        self.send_inventory_slot(kawari::common::ContainerType::Currency, slot)
            .await;
        self.set_grand_company_rank(rank + 1);
        self.send_grand_company_info().await;
        tracing::info!("Grand Company {company:?}: promoted to rank {}", rank + 1);
        true
    }

    pub fn set_grand_company_rank(&mut self, rank: u8) {
        if self.player_data.grand_company.active_company != IpcGrandCompany::None {
            let company_index = self.player_data.grand_company.active_company as usize - 1;
            self.player_data.grand_company.company_ranks.0[company_index] = rank;
            {
                let mut db = self.database.lock();
                db.commit_grand_companies(&self.player_data);
            }
        }
    }

    pub fn set_grand_company(&mut self, company: IpcGrandCompany) {
        self.player_data.grand_company.active_company = company;

        // If the player has no grand company after the above, this is a no-op.
        if self.grand_company_rank(company).unwrap_or_default() == 0 {
            self.set_grand_company_rank(1);
        }

        {
            let mut db = self.database.lock();
            db.commit_grand_companies(&self.player_data);
        }
    }

    pub async fn send_crafting_gathering_information(&mut self) {
        // gathering log
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::GatheringLog {
                bitmask: self.player_data.quest.gathered_gathering_items.data.clone(),
            });
            self.send_ipc_self(ipc).await;
        }

        // crafting log
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::CraftingLog {
                bitmask: self.player_data.quest.completed_recipes.data.clone(),
            });
            self.send_ipc_self(ipc).await;
        }

        // Which position each gathering point shows (retail sends it once at login.)
        self.send_gathering_phases().await;
    }

    pub fn check_tele_sharing_eligibility(&mut self, aetheryte_id: u32) {
        let mut game_data = self.game_data.lock();
        // The teleport can only be offered if it's a town aetheryte (not shard) and the player is actually teleporting, not using the aethernet.
        self.can_share_teleport = game_data.is_aetheryte(aetheryte_id);
    }

    pub fn can_offer_teleport(&self) -> bool {
        self.party_id != 0 && self.can_share_teleport
    }
}

/// Whether this packet puts the player somewhere: an ActorSetPos of theirs, or a gimmick jump of theirs (ActorControl
/// 220). Retail followed each with ActorControlSelf 236 at once (`position_sequence_after`.)
pub(crate) fn puts_the_player(
    data: &ServerZoneIpcData,
    source: ObjectId,
    player: ObjectId,
) -> bool {
    let theirs = source == ObjectId::default() || source == player;
    theirs
        && match data {
            ServerZoneIpcData::ActorSetPos(_) => true,
            ServerZoneIpcData::ActorControl(control) => matches!(
                control.category,
                ActorControlCategory::ExecuteGimmickJump { .. }
            ),
            _ => false,
        }
}

/// ActorControlSelf 236's count after `last`: 1 for the first after a ZoneInit, then 2, 3, 0, 1... (Bob's Chronofoil
/// captures: all 534 ActorSetPos and 61 gimmick jumps of the player's were followed by it in the same instant, the
/// first after each of 247 zone-ins was 1, and each next one the last plus one, modulo 4, 323 times.) What the client
/// does with it isn't known (upstream's `MovementRelatedUnk`, in UpdatePositionHandler.)
pub(crate) fn position_sequence_after(last: u32) -> u32 {
    (last + 1) % 4
}

#[cfg(test)]
mod position_sequence_tests {
    use super::*;
    use kawari::ipc::zone::{ActorControl, ActorSetPos};

    /// Known issues, the toss: retail sent ActorControlSelf 236 after each time it put the player somewhere (an
    /// ActorSetPos, a gimmick jump), counting 1, 2, 3, 0 from each zone-in; Kawari never sent it.
    #[test]
    fn putting_the_player_somewhere_is_counted_as_retail_s_was() {
        let me = ObjectId(0x1000_0001);
        let set_pos = ServerZoneIpcData::ActorSetPos(ActorSetPos::default());
        assert!(puts_the_player(&set_pos, ObjectId::default(), me));
        assert!(puts_the_player(&set_pos, me, me));
        assert!(!puts_the_player(&set_pos, ObjectId(0x4000_0001), me));
        let jump = ServerZoneIpcData::ActorControl(ActorControl {
            category: ActorControlCategory::ExecuteGimmickJump {
                landing_position_y: 0.0,
                landing_position_x: 0.0,
                landing_position_z: 0.0,
                gimmick_jump_type: 64,
                unk1: 0,
            },
        });
        assert!(puts_the_player(&jump, me, me));
        let mut sequence = 0;
        let counted: Vec<u32> = (0..5)
            .map(|_| {
                sequence = position_sequence_after(sequence);
                sequence
            })
            .collect();
        assert_eq!(counted, [1, 2, 3, 0, 1]);
    }
}
