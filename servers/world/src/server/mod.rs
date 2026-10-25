use glam::{Affine3A, EulerRot, Vec3A};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    env::consts::EXE_SUFFIX,
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::Receiver;

use crate::{
    GameData, Navmesh,
    lua::KawariLua,
    server::{
        action::{execute_action, handle_action_messages, use_event_item},
        actor::{
            NetworkedActor, NpcState, kill_actor, set_character_mode, set_player_minion,
            spawn_custom_bnpc, update_actor_hp_mp,
        },
        chat::handle_chat_messages,
        director::{
            DirectorData, director_tick, handle_director_messages, handle_director_trigger,
            send_director_vars,
        },
        effect::{handle_effect_messages, remove_effect, send_effects_list},
        fate::{ended_fate, fate_tick, start_fate, start_next_fate, unk10_fate},
        instance::{
            Instance, NavmeshGenerationStep, QueuedTaskData, WEATHER_WINDOW_SECONDS,
            remove_actor_from_instance,
        },
        linkshell::handle_linkshell_messages,
        network::{DestinationNetwork, NetworkState},
        party::{
            NUM_TARGET_SIGNS, get_party_id_from_actor_id, handle_party_messages,
            send_party_positions, update_party_position, update_party_waymark,
            update_party_waymarks,
        },
        social::handle_social_messages,
        spawn_allocator::SpawnAllocator,
        zone::{
            MapGimmick, change_zone_to_player, change_zone_warp_to_entrance,
            change_zone_warp_to_pop_range, do_change_zone, handle_zone_messages,
        },
    },
};
use kawari::{
    common::{
        CharacterMode, DEAD_DESPAWN_TIME, HandlerId, HandlerType, MAX_SPAWNED_ACTORS,
        MAX_SPAWNED_OBJECTS, MOB_RESPAWN_TIME, ObjectId, ObjectTypeId, ObjectTypeKind, Position,
        WarpType, determine_initial_pop_range, euler_to_direction, is_private_area,
        mob_respawn_time, timestamp_secs,
    },
    config::get_config,
    ipc::zone::{
        ActionRequest, ActionType, ActorControlCategory, BattleNpcSubKind, ClientTriggerCommand,
        ClientTriggerTarget, Condition, Conditions, DutyFinderSetting, EnmityList, Hater,
        HaterList, ObjectKind, PlayerEnmity, PrepareZoning, PrepareZoningFlag, ServerZoneIpcData,
        ServerZoneIpcSegment, WaymarkPreset, WeatherChange,
    },
};

use super::{ClientId, FromServer, ToServer};

mod action;
mod actor;
pub mod bot_friends;
mod chat;
mod companion;
pub use companion::{
    COMMAND_FOLLOW, COMMAND_HEALER, COMMAND_WITHDRAW, CompanionProfile, MAX_RANK, eat_onion,
    gain_exp,
};
mod director;
mod dodge;
mod effect;
mod glamour;
mod instance;
mod job_gauge;
mod loot;
mod modifiers;
use job_gauge::CLASSJOB_WHITE_MAGE;
mod linkshell;
mod network;
mod party;
mod party_bot;
mod party_npc;
mod periodic;
mod pet;
pub use party::{Party, PartyMember};
pub use party_bot::BOT_LEVEL;
pub use party_npc::chosen_members;
use periodic::periodic_tick;
pub use rotation::{ActionMechanics, GCD_GROUP};
mod fate;
mod first_attack;
mod npc_behavior;
mod placement;
mod rotation;
mod social;
mod spawn_allocator;
mod zone;

#[derive(Default, Debug, Clone)]
struct ClientState {
    actor_allocator: SpawnAllocator<MAX_SPAWNED_ACTORS, 1>, // Indices start at 1 because the player always takes the 0 index.
    object_allocator: SpawnAllocator<MAX_SPAWNED_OBJECTS>,
}

impl ClientState {
    /// Check if this client has spawned said `object_id`.
    pub fn has_spawned(&self, object_id: ObjectId) -> bool {
        self.actor_allocator.contains(object_id) || self.object_allocator.contains(object_id)
    }
}

#[derive(Default, Debug)]
struct WorldServer {
    instances: Vec<Instance>,
    /// Each player's chocobo companion (`companion.rs`), by their actor.
    companions: HashMap<ObjectId, companion::Companion>,
    // TODO: Eventually remove these once we can reliably and ergonomically run misc. tasks on slower intervals!
    rested_exp_counter: i32,
    party_positions_counter: i32,
}

impl WorldServer {
    /// Ensures an instance exists, and creates one if not found.
    fn ensure_exists(
        &mut self,
        zone_id: u16,
        game_data: &mut GameData,
        lua: &KawariLua,
    ) -> &mut Instance {
        let is_public_instance;
        if let Some(intended_use) = game_data.get_intended_use(zone_id as u32) {
            is_public_instance = !is_private_area(intended_use);
        } else {
            is_public_instance = true; // Fall back to assuming it's public I guess
        }

        if is_public_instance {
            // create a new instance if necessary
            if !self
                .instances
                .iter()
                .any(|x| x.zone.id == zone_id && x.content_finder_condition_id == 0)
            {
                tracing::info!("Creating new public instance for zone {zone_id}!");
                self.instances
                    .push(Instance::new(zone_id, game_data, lua, false, false));
            }

            self.instances
                .iter_mut()
                .find(|x| x.zone.id == zone_id && x.content_finder_condition_id == 0)
                .unwrap()
        } else {
            tracing::info!("Creating new private instance for zone {zone_id}!");
            self.instances
                .push(Instance::new(zone_id, game_data, lua, false, false));
            self.instances.last_mut().unwrap()
        }
    }

    /// Finds the instance associated with an actor, or returns None if they are not found.
    fn find_actor_instance(&self, actor_id: ObjectId) -> Option<&Instance> {
        self.instances
            .iter()
            .find(|instance| instance.actors.contains_key(&actor_id))
    }

    /// Finds the instance associated with an actor, or returns None if they are not found.
    fn find_actor_instance_mut(&mut self, actor_id: ObjectId) -> Option<&mut Instance> {
        self.instances
            .iter_mut()
            .find(|instance| instance.actors.contains_key(&actor_id))
    }

    /// Creates a new instance for a quest battle, which is always solo.
    fn create_instance_for_quest_battle(
        &mut self,
        zone_id: u16,
        quest_battle_id: u32,
        game_data: &mut GameData,
        server_lua: &KawariLua,
    ) -> &mut Instance {
        tracing::info!(
            "Creating new instance for quest battle {quest_battle_id} (zone {zone_id})!"
        );

        // The instance needs the server's Lua state (for FATEs, which are then removed), the director gets its own.
        let mut instance = Instance::new(zone_id, game_data, server_lua, false, false);
        let lua = KawariLua::new();
        instance.content_id = quest_battle_id as u16;

        // Quest battles often take place in a copy of an overworld zone, but nothing from the overworld happens in them:
        // everything that fights is spawned by the script.
        instance.fates.clear();
        instance.candiate_fates.clear();
        instance
            .directors
            .retain(|x| x.id.handler_type() != HandlerType::Fate);
        instance
            .queued_task
            .retain(|x| !matches!(x.data, QueuedTaskData::EndFate { .. }));
        instance
            .actors
            .retain(|_, actor| !matches!(actor, NetworkedActor::Npc { .. }));

        let id = HandlerId::new(HandlerType::QuestBattle, quest_battle_id as u16);
        let mut director = DirectorData {
            id,
            ..Default::default()
        };

        let info = game_data
            .get_quest_battle(quest_battle_id)
            .unwrap_or_default();
        instance.duration = Some(Duration::from_mins(info.time_limit as u64));
        director.quest_battle_quest_id = info.quest_id;

        // The QuestBattle's variables (the NPCs it spawns and such) are available to its script, like a quest's.
        for (name, value) in &info.variables {
            if let Err(err) = lua.0.globals().set(name.as_str(), *value) {
                tracing::warn!("Failed to set {name} for quest battle {quest_battle_id}: {err:?}");
            }
        }

        // The shared battle framework (roles, focus, phases, heals: content/lib/Battle.lua) comes first, so the
        // battle's script can describe its fight with it.
        let library = get_config()
            .filesystem
            .locate_script_file("content/lib/Battle.lua");
        if let Ok(file) = std::fs::read(&library)
            && let Err(err) = lua.0.load(file).set_name("@".to_string() + &library).exec()
        {
            tracing::warn!("Syntax error in {library}: {err:?}");
        }

        let file_name = get_config()
            .filesystem
            .locate_script_file(&format!("content/questbattle/{quest_battle_id}.lua"));
        match std::fs::read(&file_name) {
            Ok(file) => {
                if let Err(err) = lua
                    .0
                    .load(file)
                    .set_name("@".to_string() + &file_name)
                    .exec()
                {
                    tracing::warn!(
                        "Syntax error in {file_name}: {err:?} the quest battle won't be scripted!"
                    );
                } else {
                    // Where the player starts, since quest battles don't have an entrance like other content.
                    let start: Option<(Position, f32)> = (|| {
                        let position: mlua::Table = lua.0.globals().get("START_POSITION").ok()?;
                        let rotation: f32 =
                            lua.0.globals().get("START_ROTATION").unwrap_or_default();
                        Some((
                            Position(Vec3A::new(
                                position.get("x").ok()?,
                                position.get("y").ok()?,
                                position.get("z").ok()?,
                            )),
                            rotation,
                        ))
                    })();
                    instance.quest_battle_start = start;

                    director.lua = lua;
                    director.setup();
                }
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to load {file_name}: {err:?} the quest battle won't be scripted!"
                );
            }
        }

        instance.directors.push(director);
        self.instances.push(instance);
        self.instances.last_mut().unwrap()
    }

    fn create_instance_for_content(
        &mut self,
        zone_id: u16,
        content_finder_condition: u16,
        content_settings: DutyFinderSetting,
        game_data: &mut GameData,
        lua: &KawariLua,
    ) -> Option<&mut Instance> {
        tracing::info!(
            "Creating new instance for content {content_finder_condition} (zone {zone_id})!"
        );

        let mut instance = Instance::new(
            zone_id,
            game_data,
            lua,
            content_settings.contains(DutyFinderSetting::EXPLORER_MODE),
            true, // Needed for Occult Crescent I guess
        );
        instance.content_finder_condition_id = content_finder_condition;

        if let Some(director_type) = HandlerType::from_intended_use(instance.zone.intended_use) {
            instance.content_id = game_data
                .find_content_for_content_finder_id(instance.content_finder_condition_id)
                .unwrap();
            instance.duration = Some(Duration::from_mins(
                game_data
                    .find_content_time_limit(instance.content_id)
                    .unwrap_or_default() as u64,
            ));

            let id = HandlerId::new(director_type, instance.content_id);

            // Setup Lua state for our director
            let lua = KawariLua::new();

            // Find the script for this content
            let content_short_name = game_data
                .get_content_short_name(content_finder_condition)
                .unwrap();
            let file_name = get_config()
                .filesystem
                .locate_script_file(&format!("content/{content_short_name}.lua"));

            let mut director = DirectorData {
                id,
                ..Default::default()
            };

            let result = std::fs::read(&file_name);
            if let Err(err) = result {
                tracing::warn!(
                    "Failed to load {}: {:?} instance content won't be scripted!",
                    file_name,
                    err
                );
            } else {
                let file = result.unwrap();

                if let Err(err) = lua
                    .0
                    .load(file)
                    .set_name("@".to_string() + &file_name)
                    .exec()
                {
                    tracing::warn!(
                        "Syntax error in {}: {:?} instance content won't be scripted!",
                        file_name,
                        err
                    );
                } else {
                    director.lua = lua;

                    // Call into the onSetup function before returning, as we need the flag to be initialized before any players change zones.
                    director.setup();
                }
            }

            instance.directors.push(director);
        } else {
            tracing::warn!(
                "No director created for TerritoryIntendedUse {}! If this is unexpected, please report it.",
                instance.zone.intended_use
            );
        }

        // Ensure we have the entrance set correctly
        let entrance_id = game_data
            .get_content_entrance_id(content_finder_condition)
            .expect("Failed to find entrance ID?");
        for range in &mut instance.zone.map_ranges {
            if range.instance_id == entrance_id {
                range.entrance = true;
                break;
            }
        }

        instance.content_settings = Some(content_settings);

        // Set up level sync
        let needs_sync = {
            if content_settings.contains(DutyFinderSetting::UNRESTRICTED_PARTY) {
                content_settings.contains(DutyFinderSetting::LEVEL_SYNC)
            } else {
                !content_settings.contains(DutyFinderSetting::EXPLORER_MODE)
            }
        };
        if needs_sync {
            instance.synced_level =
                game_data.find_content_synced_level(instance.content_finder_condition_id);
            instance.synced_item_level =
                game_data.find_content_synced_item_level(instance.content_finder_condition_id);
        }

        self.instances.push(instance);

        self.instances.last_mut()
    }

    fn find_actor_by_name(&self, name: &str) -> ObjectId {
        for instance in &self.instances {
            for (id, actor) in &instance.actors {
                if let Some(spawn) = actor.get_player_spawn()
                    && spawn.common.name == name
                {
                    return *id;
                }
            }
        }

        ObjectId::default()
    }

    /// Removes instances without players in them, which wastes resources.
    ///
    /// An instance whose navmesh is still being generated is kept too, so the result gets loaded. (This used to be the
    /// other way around: an instance with a navmesh generating was removed even with players in it, which left them
    /// stuck on the loading screen at their next zone change.)
    fn cleanup_dead_instances(&mut self) {
        self.instances.retain(|instance| {
            matches!(
                instance.generate_navmesh,
                NavmeshGenerationStep::Started(..)
            ) || instance
                .actors
                .iter()
                .any(|x| matches!(x.1, NetworkedActor::Player { .. }))
        });
    }
}

/// Moving during the last half second of a cast doesn't interrupt it ("slidecasting"), like retail.
const SLIDECAST_WINDOW_MS: u64 = 500;
/// How often the server logic runs, in milliseconds.
const SERVER_TICK_MILLISECONDS: u32 = 500;

/// How often players regenerate HP and MP, and how much MP (retail, from Bob's Chronofoil captures.)
const PLAYER_REGEN_INTERVAL: Duration = Duration::from_secs(3);
const MP_REGEN_IN_COMBAT: u32 = 200;
const MP_REGEN_OUT_OF_COMBAT: u32 = 600;

/// GP a gatherer gets back every tick away from a node: 5, one more each with the Enhanced GP Regeneration traits at
/// 70, 80 and 83 (Trait sheet Values 1/2/3; GatherBuddy Reborn's model. The traits' quests aren't checked.)
fn gp_regen(level: u8) -> u16 {
    match level {
        83.. => 8,
        80.. => 7,
        70.. => 6,
        _ => 5,
    }
}

/// How often queued tasks (finished casts, event actions, despawns...) are checked, in milliseconds. Much finer than
/// the logic tick, so a cast resolves when its cast bar ends rather than up to half a second later.
const TASK_TICK_MILLISECONDS: u32 = 20;

/// A player's enmity on an enemy's list as a percentage of the top (floor), as the HaterList shows it.
fn enmity_percent(enemy: &NetworkedActor, player: ObjectId) -> u32 {
    let NetworkedActor::Npc { enmity, .. } = enemy else {
        return 0;
    };
    let top = enmity.values().copied().max().unwrap_or(0);
    let theirs = enmity.get(&player).copied().unwrap_or(0);
    if top == 0 {
        0
    } else {
        (theirs as u64 * 100 / top as u64) as u32
    }
}

/// An enemy's list as the EnmityList shows it: the players on it, highest first, as percentages of the top.
fn target_enmity_list(enemy: &NetworkedActor) -> EnmityList {
    let NetworkedActor::Npc { enmity, .. } = enemy else {
        return EnmityList::default();
    };
    let top = enmity.values().copied().max().unwrap_or(0);
    let mut list: Vec<PlayerEnmity> = enmity
        .iter()
        .map(|(actor_id, amount)| PlayerEnmity {
            actor_id: *actor_id,
            enmity: if top == 0 {
                0
            } else {
                (*amount as u64 * 100 / top as u64) as u32
            },
        })
        .collect();
    list.sort_by(|a, b| {
        b.enmity
            .cmp(&a.enmity)
            .then(a.actor_id.0.cmp(&b.actor_id.0))
    });
    list.truncate(8);
    EnmityList {
        count: list.len() as u32,
        list,
    }
}

/// Status: a revive offered by Raise and the like (see `EffectsBuilder::offer_revive`.)
const STATUS_REVIVE_OFFER: u16 = 148;
/// ClientTrigger 200's choice for accepting a Raise (8 is returning to the starting point.)
const REVIVE_ACCEPT_RAISE: u32 = 5;
/// Weakness and Brink of Death (after a second raise while weakened), 100 s each, and the 5 s status retail gives a
/// raised player (418, gone on their first action.)
const STATUS_WEAKNESS: u16 = 43;
const STATUS_BRINK_OF_DEATH: u16 = 44;
const STATUS_RAISED: u16 = 418;
const WEAKNESS_DURATION: f32 = 100.0;
const RAISED_DURATION: f32 = 5.0;

/// Brings a dead player back after they accepted a Raise, as retail (Bob's Chronofoil captures, 19 raises): 20 % of
/// their HP and MP, Weakness (or Brink of Death if already weakened) and status 418, through the same warp a revive
/// uses (PrepareZoning with the raise's effect 181, warp type 5) to their raiser (`party_bot::raiser_position`; where
/// they lie if the raiser has gone.)
fn revive_in_place(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    from_id: ClientId,
    actor_id: ObjectId,
) {
    let beside = party_bot::raiser_position(instance, actor_id);
    let Some(actor) = instance.find_actor_mut(actor_id) else {
        return;
    };
    let weakened = actor
        .status_effects()
        .is_some_and(|effects| effects.get(STATUS_WEAKNESS).is_some());
    if let Some(effects) = actor.status_effects_mut() {
        effects.remove(STATUS_REVIVE_OFFER);
        effects.remove(STATUS_WEAKNESS);
    }
    let common = actor.get_common_spawn_mut();
    common.health_points = (common.max_health_points / 5).max(1);
    common.resource_points = common.max_resource_points / 5;
    let position = beside.unwrap_or(common.position);
    common.position = position;
    let rotation = common.rotation;

    let penalty = if weakened {
        STATUS_BRINK_OF_DEATH
    } else {
        STATUS_WEAKNESS
    };
    for (status, duration) in [
        (penalty, WEAKNESS_DURATION),
        (STATUS_RAISED, RAISED_DURATION),
    ] {
        crate::server::effect::gain_effect_instance(
            network.clone(),
            from_id,
            instance,
            actor_id,
            status,
            0,
            duration,
            ObjectId(0xE000_0000),
            true,
        );
    }
    update_actor_hp_mp(network.clone(), instance, actor_id);

    let mut network = network.lock();
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::PrepareZoning(PrepareZoning {
        territory_type_id: instance.zone.id,
        warp_type: WarpType::Unk5,
        vfx_id: 181,
        hide_character: 2,
        fade_out_delay: 3,
        flags: PrepareZoningFlag::UNK2,
        ..Default::default()
    }));
    network.send_to_by_actor_id(
        actor_id,
        FromServer::PacketSegment(ipc, actor_id),
        DestinationNetwork::ZoneClients,
    );
    do_change_zone(
        &mut network,
        instance,
        false,
        Some(position),
        Some(rotation),
        from_id,
        WarpType::Unk5,
    );
}

/// How long natural weather changes fade in, in seconds (retail's WeatherId: 20.0.)
const WEATHER_FADE_SECONDS: f32 = 20.0;

/// A player's time between auto-attacks, in milliseconds: their weapon's delay sped up or slowed by haste (2 s if
/// they have no weapon.)
fn auto_attack_delay_ms(parameters: &crate::zone_connection::BaseParameters) -> i64 {
    let delay = if parameters.weapon_delay_ms == 0 {
        2000
    } else {
        parameters.weapon_delay_ms
    };
    (delay as i64 * parameters.haste.max(1) as i64 / 100).max(500)
}

fn server_logic_tick(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    lua: Arc<Mutex<KawariLua>>,
) {
    let mut actors_to_update_hp_mp = Vec::new();
    let mut actors_to_fake_zone_jump = Vec::new();

    {
        let mut data = data.lock();
        let rested_exp_counter = data.rested_exp_counter;
        let party_positions_counter = data.party_positions_counter;

        data.cleanup_dead_instances();
        // Chocobo companions: their time, leaving, and following their owner into fights.
        companion::update(&mut data, &network);

        // Send a periodic update to all parties about where their members are in the world.
        // TODO: On retail this is sent once every 5 seconds, so sending this at a slower interval would be more ideal.
        if party_positions_counter == 0 {
            let mut network = network.lock();
            send_party_positions(&mut network);
        }

        for instance in &mut data.instances {
            // Party bots decide what to do before the NPCs move.
            party_bot::update(&network, &game_data, instance);
            // Pets left far behind (a teleport) go back beside their owner before they walk.
            pet::catch_up(&mut network.lock(), instance);
            // Bots choose for coffers' items, and items whose time is up are decided.
            loot::tick(&mut network.lock(), instance);
            // BNpcs a director spawned hidden come into view.
            director::update_appearances(&mut network.lock(), instance, std::time::Instant::now());
            let mut haters = HashMap::new();
            npc_behavior::npc_behavior(network.clone(), game_data.clone(), instance, &mut haters);

            let mut actors_now_gimmick_jumping = Vec::new();
            let mut actors_now_inside_instance_exits = Vec::new();
            let mut actors_now_outside_instance_entrances = Vec::new();
            let mut actors_autoattacking = Vec::new();
            let mut combat_over = Vec::new();
            let mut enmity_updates = Vec::new();

            // Player area stuffs
            for (id, actor) in &instance.actors {
                // Only check players
                let NetworkedActor::Player {
                    conditions,
                    executing_gimmick_jump,
                    inside_instance_exit: inside_instance_entrance,
                    autoattack_target,
                    autoattack_timing,
                    parameters,
                    ..
                } = actor
                else {
                    continue;
                };

                // The enmity UI as retail fills it (Bob's Chronofoil captures, 39,248 lists): the HaterList has each
                // enemy that has the player on its list with the player's enmity as a percentage of its top; the
                // EnmityList is the player's target's list (players only, at most 8, highest first, the top at 100.)
                // Sent only when they change (below); they used to go out every half second with everything at 100.
                let hater_list = {
                    let mut list: Vec<Hater> = haters
                        .get(id)
                        .map(|haters| {
                            haters
                                .iter()
                                .map(|enemy| Hater {
                                    actor_id: *enemy,
                                    enmity: instance
                                        .find_actor(*enemy)
                                        .map(|enemy| enmity_percent(enemy, *id))
                                        .unwrap_or(0),
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    // In the order they were last sent, newcomers after: retail's list kept its order as enemies
                    // joined (the capture above), and the enemies here come in no particular order each tick.
                    if let Some((last, _)) = instance.last_enmity_lists.get(id) {
                        let place = |actor: &ObjectId| {
                            last.list
                                .iter()
                                .position(|hater| hater.actor_id == *actor)
                                .unwrap_or(usize::MAX)
                        };
                        list.sort_by_key(|hater| (place(&hater.actor_id), hater.actor_id.0));
                    } else {
                        list.sort_by_key(|hater| hater.actor_id.0);
                    }
                    list.truncate(32);
                    HaterList {
                        count: list.len() as u32,
                        list,
                    }
                };
                let enmity_list = instance
                    .find_actor(actor.get_common_spawn().target_id.object_id)
                    .map(target_enmity_list)
                    .unwrap_or_default();
                enmity_updates.push((*id, hater_list, enmity_list));

                // Find the ClientState for this player.
                let mut network = network.lock();

                let Some((handle, state)) = network.get_by_actor_mut(*id) else {
                    continue;
                };

                // TODO: find a better place for this
                if autoattack_target.is_some() {
                    // Schedule any pending auto-attacks: one every weapon delay (`autoattack_timing` counts
                    // milliseconds, see the regeneration loop), on the player's current target when it's in reach
                    // (`auto_attack_target`); otherwise the swing waits, ready, until it is.
                    let should_auto_attack = *autoattack_timing >= auto_attack_delay_ms(parameters);
                    if should_auto_attack
                        && let Some(target) = action::auto_attack_target(instance, *id)
                    {
                        actors_autoattacking.push((*id, target));
                    }
                }

                // Check for overlapping map ranges
                let overlapping_ranges =
                    instance.zone.get_overlapping_map_ranges(actor.position().0);
                let in_sanctuary = overlapping_ranges.iter().filter(|x| x.sanctuary).count() > 0;

                // We're on the 10 second mark, and you're in a sanctuary...
                if in_sanctuary && rested_exp_counter == 0 {
                    // Update rested EXP! (This means it only has a ten second granularity, but who cares?)
                    let msg = FromServer::IncrementRestedExp;
                    if handle.send(msg).is_err() {
                        // TODO: remove as needed
                        //self.to_remove.push(id);
                    }
                }

                let mut inside_any_instance_entrances = false;

                // Process gimmicks
                if !executing_gimmick_jump {
                    for range in &overlapping_ranges {
                        if let Some(gimmick) = &range.gimmick {
                            match gimmick {
                                MapGimmick::Generic => {
                                    if let Some(director) = &mut instance.directors.first_mut() {
                                        director.on_gimmick_rect(range.instance_id);
                                    }
                                }
                                MapGimmick::Jump {
                                    to_position,
                                    gimmick_jump_type,
                                    sgb_animation_id,
                                    eobj_instance_id,
                                } => {
                                    // Tell the client to execute the gimmick jump
                                    let msg = FromServer::ActorControlSelf(
                                        ActorControlCategory::ExecuteGimmickJump {
                                            landing_position_x: to_position.x,
                                            landing_position_y: to_position.y,
                                            landing_position_z: to_position.z,
                                            gimmick_jump_type: *gimmick_jump_type,
                                            unk1: 0,
                                        },
                                    );
                                    actors_now_gimmick_jumping.push(*id);
                                    if handle.send(msg).is_err() {
                                        // TODO: remove as needed
                                        //self.to_remove.push(id);
                                    }

                                    // Play the corresponding animation for the EObj
                                    if let Some(eobj) = instance.find_object(*eobj_instance_id) {
                                        let msg = FromServer::ActorControl(
                                            eobj,
                                            ActorControlCategory::PlaySharedGroupTimeline {
                                                timeline_id: *sgb_animation_id,
                                            },
                                        );
                                        if handle.send(msg).is_err() {
                                            // TODO: remove as needed
                                            //self.to_remove.push(id);
                                        }
                                    }
                                }
                                MapGimmick::FakeExit { exit_pop_range_id } => {
                                    actors_to_fake_zone_jump.push((*id, *exit_pop_range_id));
                                }
                            }
                        }
                        if range.entrance {
                            inside_any_instance_entrances = true;
                            if !inside_instance_entrance {
                                let msg =
                                    FromServer::EnteredInstanceEntranceRange(range.instance_id);
                                actors_now_inside_instance_exits.push(*id);
                                if handle.send(msg).is_err() {
                                    // TODO: remove as needed
                                    //self.to_remove.push(id);
                                }
                            }
                        }
                    }
                }

                if !inside_any_instance_entrances {
                    actors_now_outside_instance_entrances.push(*id);
                }

                // Retail's InCombat condition, while any enemy is after them: the client won't Teleport, Return, mount
                // or start talking to NPCs while it's set, and it's what the HP/MP regeneration below waits on.
                let in_combat = haters.get(id).is_some_and(|haters| !haters.is_empty());
                if in_combat != conditions.has_condition(Condition::InCombat) {
                    let _ =
                        handle.send(FromServer::ToggleCondition(Condition::InCombat, in_combat));
                    if !in_combat {
                        combat_over.push(*id);
                    }
                }

                let is_in_duel_area = overlapping_ranges.iter().filter(|x| x.duel).count() > 0;
                let has_duel_condition = conditions.has_condition(Condition::InDuelingArea);

                if is_in_duel_area != has_duel_condition {
                    // Update conditions
                    {
                        let mut conditions = *conditions;
                        conditions.toggle_condition(Condition::InDuelingArea, is_in_duel_area);

                        let msg = FromServer::Conditions(conditions);
                        if handle.send(msg).is_err() {
                            // TODO: remove as needed
                            //self.to_remove.push(id);
                        }
                    }

                    // Send log message
                    {
                        let log_message = if is_in_duel_area {
                            2692 // Duels permitted in current area.
                        } else {
                            2693 // Duels not permitted in current area.
                        };

                        let msg = FromServer::ActorControlSelf(ActorControlCategory::LogMessage {
                            log_message,
                            id: 0,
                        });
                        if handle.send(msg).is_err() {
                            // TODO: remove as needed
                            //self.to_remove.push(id);
                        }
                    }
                }

                // We want to prioritize actors closest to us!
                let mut actors_by_distance = instance.actors.iter().collect::<Vec<_>>();
                actors_by_distance.sort_by(|a, b| {
                    let a_position = a.1.position();
                    let b_position = b.1.position();

                    let a_distance = Vec3A::distance(actor.position().0, a_position.0);
                    let b_distance = Vec3A::distance(actor.position().0, b_position.0);

                    a_distance.total_cmp(&b_distance)
                });
                let mut aoe_actors_to_evaluate = Vec::new();
                for (other_id, other_actor) in actors_by_distance {
                    // We're always in our own view
                    if *id == *other_id {
                        continue;
                    }

                    // If the actor isn't valid, don't bother spawning yet.
                    if !other_actor.is_valid() {
                        continue;
                    }

                    // If the actor _should_ be in the view of the other.
                    let in_range = actor.in_range_of(other_actor);
                    let has_been_spawned = state.has_spawned(*other_id);

                    // There are four states:
                    // Walked out = (Has been spawned, no longer in range)
                    // Walked in = (Hasn't been spawned, in range)
                    // Still in = (Has been spawned, in range)
                    // Still out = (Hasn't been spawned, not in range)

                    let walked_out = has_been_spawned && !in_range;
                    let walked_in = !has_been_spawned && in_range;
                    let still_in = has_been_spawned && in_range;
                    let still_out = !has_been_spawned && !in_range;

                    if walked_out {
                        if let Some(spawn_index) = state.actor_allocator.free(*other_id) {
                            let msg = FromServer::DeleteActor(*other_id, spawn_index);

                            if handle.send(msg).is_err() {
                                // TODO: remove as needed
                                //self.to_remove.push(id);
                            }
                        } else if let Some(spawn_index) = state.object_allocator.free(*other_id) {
                            let msg = FromServer::DeleteObject(spawn_index);

                            if handle.send(msg).is_err() {
                                // TODO: remove as needed
                                //self.to_remove.push(id);
                            }
                        }
                    } else if walked_in {
                        // Spawn this actor
                        if let Some(msg) = NetworkState::spawn_existing_actor_message(
                            state,
                            *other_id,
                            other_actor,
                        ) {
                            if handle.send(msg).is_err() {
                                // TODO: remove as needed
                                //self.to_remove.push(id);
                            }
                        } else {
                            // Early exit if the client refuses to spawn any more actors
                            continue;
                        }
                    } else if still_in || still_out {
                        // Do nothing
                    } else {
                        unreachable!();
                    }

                    if still_in
                        && let NetworkedActor::Npc { state, .. } = other_actor
                        && *state == NpcState::AoeTest
                    {
                        aoe_actors_to_evaluate.push(*other_id);
                    }
                }

                // Process AoEs (currently only test actors) and a cone at that
                for aoe_actor_id in &aoe_actors_to_evaluate {
                    let Some(aoe_actor) = instance.find_actor(*aoe_actor_id) else {
                        continue;
                    };

                    let our_position = actor.position().0;
                    let their_position = aoe_actor.position().0;
                    let their_rotation = aoe_actor.rotation();

                    let max_cone_distance = 12.0;
                    let max_cone_angle = 60.0;

                    let mut pass =
                        Vec3A::distance(our_position, their_position) < max_cone_distance;

                    let direction_to_target = their_position - our_position;
                    let forward = Vec3A::new(0.0, 0.0, -1.0).rotate_y(their_rotation);
                    let cone_angle =
                        Vec3A::angle_between(forward, direction_to_target).to_degrees();
                    if pass && cone_angle > max_cone_angle {
                        pass = false;
                    }

                    let lockon_id = if pass {
                        136 // green checkmark
                    } else {
                        137 // red cross
                    };

                    let msg = FromServer::ActorControlSelf(ActorControlCategory::ShowLockonVfx {
                        id: lockon_id,
                        target: handle.actor_id,
                    });
                    if handle.send(msg).is_err() {
                        // TODO: remove as needed
                        //self.to_remove.push(id);
                    }
                }
            }

            // Send each player's enmity lists when they've changed (an empty HaterList once as their fight ends.)
            for (id, hater_list, enmity_list) in enmity_updates {
                let last = instance.last_enmity_lists.get(&id);
                let hater_changed = last.is_none_or(|(haters, _)| *haters != hater_list)
                    && !(last.is_none() && hater_list.list.is_empty());
                let enmity_changed = last.is_none_or(|(_, enmity)| *enmity != enmity_list)
                    && !(last.is_none() && enmity_list.list.is_empty());
                let mut network = network.lock();
                if hater_changed {
                    let ipc =
                        ServerZoneIpcSegment::new(ServerZoneIpcData::HaterList(hater_list.clone()));
                    network.send_to_by_actor_id(
                        id,
                        FromServer::PacketSegment(ipc, id),
                        DestinationNetwork::ZoneClients,
                    );
                }
                if enmity_changed {
                    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::EnmityList(
                        enmity_list.clone(),
                    ));
                    network.send_to_by_actor_id(
                        id,
                        FromServer::PacketSegment(ipc, id),
                        DestinationNetwork::ZoneClients,
                    );
                }
                instance
                    .last_enmity_lists
                    .insert(id, (hater_list, enmity_list));
            }

            // Their fight is over: nobody claims them any more (FirstAttack cleared, see first_attack.rs.)
            for id in combat_over {
                first_attack::clear(&mut network.lock(), instance, id);
            }

            // Process any auto-attacks
            for (actor, target) in &actors_autoattacking {
                let Some(NetworkedActor::Player { spawn, .. }) = instance.find_actor_mut(*actor)
                else {
                    continue;
                };

                let request = ActionRequest {
                    action_id: 7, // TODO: Unsure if this is actually the correct player auto-attack
                    action_type: ActionType::Action,
                    rotation1: spawn.common.rotation,
                    // Retail's auto-attacks name their target with no object type (Bob's Chronofoil captures, all
                    // 663); Kawari's said EObjOrNpc (1).
                    target: ObjectTypeId {
                        object_id: *target,
                        object_type: ObjectTypeKind::None,
                    },
                    ..Default::default()
                };

                instance.insert_task(
                    ClientId::default(),
                    *actor,
                    Duration::from_secs_f32(0.0),
                    QueuedTaskData::CastAction {
                        request: request.clone(),
                        interruptible: false,
                    },
                );
            }

            // Set players as gimmick jumping, as the client does *not* send position updates during it.
            for actor in &actors_now_gimmick_jumping {
                let Some(NetworkedActor::Player {
                    executing_gimmick_jump,
                    ..
                }) = instance.find_actor_mut(*actor)
                else {
                    continue;
                };

                *executing_gimmick_jump = true;
            }

            // TODO: we probably need a better "we just entered this maprect" event instead of this
            for actor in &actors_now_inside_instance_exits {
                let Some(NetworkedActor::Player {
                    inside_instance_exit,
                    ..
                }) = instance.find_actor_mut(*actor)
                else {
                    continue;
                };

                *inside_instance_exit = true;
            }

            for actor in &actors_now_outside_instance_entrances {
                let Some(NetworkedActor::Player {
                    inside_instance_exit,
                    ..
                }) = instance.find_actor_mut(*actor)
                else {
                    continue;
                };

                *inside_instance_exit = false;
            }

            // Job gauges that fill over time (e.g. White Mage Lilies.)
            {
                let mut gauge_updates = Vec::new();
                for (id, actor) in &mut instance.actors {
                    if let NetworkedActor::Player {
                        spawn, job_gauge, ..
                    } = actor
                        && spawn.common.class_job == CLASSJOB_WHITE_MAGE
                        && spawn.common.health_points > 0
                        && job_gauge.tick_white_mage(
                            SERVER_TICK_MILLISECONDS,
                            haters.contains_key(id),
                            spawn.common.level,
                        )
                    {
                        gauge_updates.push((*id, job_gauge.to_ipc(spawn.common.class_job)));
                    }
                }

                if !gauge_updates.is_empty() {
                    let mut network = network.lock();
                    for (id, ipc) in gauge_updates {
                        network.send_to_by_actor_id(
                            id,
                            FromServer::PacketSegment(ipc, id),
                            DestinationNetwork::ZoneClients,
                        );
                    }
                }
            }

            // Damage/healing over time
            periodic_tick(network.clone(), instance);

            // A running duty's time left is re-sent every 300 s, as retail.
            director::resync_duty_clock(&mut network.lock(), instance);

            // Unspoiled, legendary and ephemeral gathering points come and go with the Eorzea time.
            instance.update_timed_gathering_points(&mut network.lock(), timestamp_secs() as u64);

            // Out in the world the weather moves on at each 1400 s boundary, sent with retail's 20 s fade.
            let weather_window = timestamp_secs() / WEATHER_WINDOW_SECONDS;
            if weather_window != instance.weather_window
                && instance.content_id == 0
                && instance.content_finder_condition_id == 0
            {
                instance.weather_window = weather_window;
                let weather = game_data
                    .lock()
                    .get_weather(instance.zone.id as u32)
                    .unwrap_or_default() as u16;
                if weather != instance.weather_id {
                    instance.weather_id = weather;
                    let ipc =
                        ServerZoneIpcSegment::new(ServerZoneIpcData::WeatherId(WeatherChange {
                            weather_id: weather as u8,
                            fade_length: WEATHER_FADE_SECONDS,
                            ..Default::default()
                        }));
                    let players: Vec<ObjectId> = instance
                        .actors
                        .iter()
                        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                        .map(|(id, _)| *id)
                        .collect();
                    let mut network = network.lock();
                    for player in players {
                        network.send_to_by_actor_id(
                            player,
                            FromServer::PacketSegment(ipc.clone(), player),
                            DestinationNetwork::ZoneClients,
                        );
                    }
                }
            }

            // Players regenerate on retail's 3 s tick (Bob's Chronofoil captures; every point exact, levels 6 to 90): HP by
            // floor(max / 100) in combat and floor(max / 10) out of it; MP by 200 + floor(150 × (Piety − the level's main
            // stat) / the level's divisor) in combat and 600 out of it. (Retail ticks each actor on its own 3 s phase;
            // here one timer per instance.) It used to run every server tick (500 ms) and not at all in combat, so
            // casters ran dry mid-fight and couldn't cast again until it ended (2026-10-17).
            let now = Instant::now();
            let regen_now = instance.next_regen_tick.is_none_or(|next| now >= next);
            // Idle enemies only regenerate out in the world (in content scripts set their health.)
            let instance_is_open_world =
                instance.content_id == 0 && instance.content_finder_condition_id == 0;
            if regen_now {
                instance.next_regen_tick = Some(now + PLAYER_REGEN_INTERVAL);
            }
            for (id, actor) in &mut instance.actors {
                if let NetworkedActor::Player {
                    spawn,
                    autoattack_timing,
                    autoattack_target,
                    parameters,
                    ..
                } = actor
                {
                    let in_combat = haters.contains_key(id);
                    let is_dead = spawn.common.health_points == 0;

                    // Auto-attacks come every weapon delay × haste / 100, as retail's (Bob's Chronofoil captures:
                    // Paladin 2.27 s for a 2.24 s weapon, Warrior 3.38-3.40 for 3.36, Bob's 2.41 for 2.4); they were
                    // every 2 s for everyone. A swing just made uses up one delay; one that couldn't be made (out of
                    // reach, casting, no enemy targeted) stays ready, so it comes as soon as it can, as retail's do
                    // (0.1 s after stepping back in). While not attacking it's ready at once (the first swing comes
                    // straight away.)
                    let delay = auto_attack_delay_ms(parameters);
                    if autoattack_target.is_some() {
                        if actors_autoattacking.iter().any(|(actor, _)| actor == id) {
                            *autoattack_timing -= delay;
                        }
                        if *autoattack_timing < delay {
                            *autoattack_timing += SERVER_TICK_MILLISECONDS as i64;
                        }
                    } else {
                        *autoattack_timing =
                            (*autoattack_timing + SERVER_TICK_MILLISECONDS as i64).min(delay);
                    }

                    if !regen_now || is_dead {
                        continue;
                    }

                    if in_combat {
                        let mut updated = false;
                        if spawn.common.resource_points < spawn.common.max_resource_points {
                            // Piety's bonus: 150 per level divisor above the level's main stat (exact for all 17 of
                            // Bob's points, e.g. 46 Piety at 14 → 203, 440 synced to 50 → 304.)
                            let piety_bonus =
                                parameters.piety.saturating_sub(parameters.level_main) * 150
                                    / parameters.level_div.max(1);
                            let amount = (MP_REGEN_IN_COMBAT + piety_bonus) as u16;
                            spawn.common.resource_points = spawn
                                .common
                                .resource_points
                                .saturating_add(amount)
                                .min(spawn.common.max_resource_points);
                            updated = true;
                        }
                        if spawn.common.health_points < spawn.common.max_health_points {
                            spawn.common.health_points = (spawn.common.health_points
                                + spawn.common.max_health_points / 100)
                                .min(spawn.common.max_health_points);
                            updated = true;
                        }
                        if updated {
                            actors_to_update_hp_mp.push(*id);
                        }
                        continue;
                    }

                    let mut updated = false;
                    if spawn.common.health_points != spawn.common.max_health_points {
                        let amount = spawn.common.max_health_points / 10;
                        spawn.common.health_points = u32::clamp(
                            spawn.common.health_points + amount,
                            0,
                            spawn.common.max_health_points,
                        );
                        updated = true;
                    }

                    // CP doesn't come back during a synthesis, nor GP over time at a node (retail; CP fills up
                    // when a synthesis ends instead, `events/crafting.rs`.)
                    let busy = matches!(
                        spawn.common.mode,
                        CharacterMode::Crafting | CharacterMode::Gathering
                    );
                    if spawn.common.resource_points != spawn.common.max_resource_points && !busy {
                        let amount = if crate::common::is_gatherer(spawn.common.class_job) {
                            gp_regen(spawn.common.level)
                        } else {
                            MP_REGEN_OUT_OF_COMBAT as u16
                        };
                        spawn.common.resource_points = u16::clamp(
                            spawn.common.resource_points + amount,
                            0,
                            spawn.common.max_resource_points,
                        );
                        updated = true;
                    }

                    if updated {
                        actors_to_update_hp_mp.push(*id);
                    }
                } else if let NetworkedActor::Npc {
                    spawn,
                    enmity,
                    state,
                    ..
                } = actor
                    && regen_now
                    && enmity.is_empty()
                    && *state != NpcState::Dead
                    && *state != NpcState::Hate
                    && spawn.common.health_points > 0
                    && spawn.common.health_points < spawn.common.max_health_points
                    && ((spawn.common.object_kind
                        == ObjectKind::BattleNpc(BattleNpcSubKind::Enemy)
                        && !spawn.common.owner_id.is_valid()
                        && instance_is_open_world)
                        // Party bots too, between fights, anywhere (a guess: players' own regeneration isn't this.)
                        || instance.party_bots.contains_key(id))
                {
                    // Enemies out of combat (or going home after giving up a chase) get 10 % of their max HP back
                    // every 3 s (Bob's Chronofoil captures, 149 samples.)
                    spawn.common.health_points = (spawn.common.health_points
                        + spawn.common.max_health_points / 10)
                        .min(spawn.common.max_health_points);
                    actors_to_update_hp_mp.push(*id);
                }
            }

            // generate navmesh if necessary
            match &instance.generate_navmesh {
                NavmeshGenerationStep::None => {}
                NavmeshGenerationStep::Needed(nvm_path) => {
                    tracing::info!(
                        "Missing navmesh {nvm_path:?}, we are going to generate it in the background now..."
                    );

                    let mut dir = std::env::current_exe().unwrap();
                    dir.pop();
                    dir.push(format!("kawari-navimesh{EXE_SUFFIX}"));

                    // start navimesh generator
                    match Command::new(dir)
                        .arg(instance.zone.id.to_string())
                        .arg(nvm_path)
                        .spawn()
                    {
                        Ok(_) => {
                            instance.generate_navmesh =
                                NavmeshGenerationStep::Started(nvm_path.clone())
                        }
                        Err(err) => {
                            tracing::error!(
                                "Unable to run kawari-navimesh due to the following error: {err}"
                            );
                            instance.generate_navmesh = NavmeshGenerationStep::None;
                        }
                    }
                }
                NavmeshGenerationStep::Started(nvm_path) => {
                    if let Ok(nvm_bytes) = std::fs::read(nvm_path) {
                        if let Some(navmesh) = Navmesh::from_existing(&nvm_bytes) {
                            instance.navmesh = navmesh;

                            tracing::info!("Successfully loaded navimesh from {nvm_path:?}");
                        } else {
                            tracing::warn!(
                                "Failed to read {nvm_path:?}, monsters will not function correctly!"
                            );
                        }
                        instance.generate_navmesh = NavmeshGenerationStep::None;
                    }
                }
            }

            // Process any director tasks for this instance.
            director_tick(network.clone(), instance, game_data.clone());
            fate_tick(network.clone(), instance, game_data.clone());
        }
        // Ensure the rested EXP counter only happens every 10 seconds.
        data.rested_exp_counter += 1;
        if data.rested_exp_counter == 21 {
            data.rested_exp_counter = 0;
        }

        // Ensure the party positions counter only happens approx. every 5 seconds.
        data.party_positions_counter += 1;
        if data.party_positions_counter == 11 {
            data.party_positions_counter = 0;
        }
    }

    for id in actors_to_update_hp_mp {
        let mut data = data.lock();
        let instance = data.find_actor_instance_mut(id).unwrap();
        update_actor_hp_mp(network.clone(), instance, id);
    }

    for (id, exit_pop_range_id) in actors_to_fake_zone_jump {
        let mut game_data = game_data.lock();
        let mut data = data.lock();
        let mut network = network.lock();
        let lua = lua.lock();
        let from_id = network.find_by_actor(id).unwrap();
        change_zone_warp_to_pop_range(
            &mut data,
            &mut network,
            &mut game_data,
            &lua,
            None, // None here means that we don't want to change their current instance
            exit_pop_range_id,
            id,
            from_id,
            WarpType::Event,
            0,
        );
    }
}

pub async fn server_main_loop(
    game_data: GameData,
    parties: HashMap<u64, Party>,
    linkshells: HashMap<u64, Vec<ObjectId>>,
    mut recv: Receiver<ToServer>,
) -> Result<(), std::io::Error> {
    let data = Arc::new(Mutex::new(WorldServer::default()));
    let network = Arc::new(Mutex::new(NetworkState {
        parties,
        linkshells,
        ..Default::default()
    }));
    let game_data = Arc::new(Mutex::new(game_data));
    let lua = Arc::new(Mutex::new(KawariLua::new()));

    // Run Init.lua and set up other Lua state
    {
        let mut lua = lua.lock();
        if let Err(err) = lua.init(game_data.clone()) {
            tracing::warn!("Failed to load Init.lua: {:?}", err);
        }
    }

    {
        let data = data.clone();
        let network = network.clone();
        let game_data = game_data.clone();
        let lua = lua.clone();
        tokio::task::spawn(async move {
            // Queued tasks run every TASK_TICK_MILLISECONDS, the general logic every SERVER_TICK_MILLISECONDS. Be careful
            // when changing the latter, as the rested EXP may become whacky.
            let mut interval =
                tokio::time::interval(Duration::from_millis(TASK_TICK_MILLISECONDS as u64));
            interval.tick().await;
            let ticks_per_logic_tick = SERVER_TICK_MILLISECONDS / TASK_TICK_MILLISECONDS;
            let mut ticks_until_logic = ticks_per_logic_tick;
            loop {
                interval.tick().await;

                // Execute general server logic
                ticks_until_logic -= 1;
                if ticks_until_logic == 0 {
                    ticks_until_logic = ticks_per_logic_tick;
                    server_logic_tick(
                        data.clone(),
                        network.clone(),
                        game_data.clone(),
                        lua.clone(),
                    );
                }

                // Execute list of queued tasks
                {
                    let mut tasks_to_execute = Vec::new();

                    // Gather list of tasks to execute
                    {
                        let mut data = data.lock();
                        for (i, instance) in data.instances.iter_mut().enumerate() {
                            for task in &instance
                                .queued_task
                                .extract_if(.., |x| x.point <= Instant::now())
                                .collect::<Vec<_>>()
                            {
                                tasks_to_execute.push((i, task.clone()));
                            }
                        }
                    }

                    for (instance_index, task) in &tasks_to_execute {
                        match &task.data {
                            QueuedTaskData::CastAction { request, .. }
                                if request.action_type == ActionType::EventItem =>
                            {
                                use_event_item(
                                    network.clone(),
                                    data.clone(),
                                    game_data.clone(),
                                    task.from_id,
                                    task.from_actor_id,
                                    request,
                                );
                            }
                            QueuedTaskData::CastAction { request, .. } => {
                                execute_action(
                                    network.clone(),
                                    data.clone(),
                                    game_data.clone(),
                                    lua.clone(),
                                    task.from_id,
                                    task.from_actor_id,
                                    request.clone(),
                                );
                            }
                            QueuedTaskData::LoseStatusEffect {
                                effect_id,
                                effect_param,
                                effect_source_actor_id,
                            } => {
                                remove_effect(
                                    network.clone(),
                                    data.clone(),
                                    lua.clone(),
                                    task.from_id,
                                    task.from_actor_id,
                                    *effect_id,
                                    *effect_param,
                                    *effect_source_actor_id,
                                );
                            }
                            QueuedTaskData::DeadFadeOut {
                                actor_id,
                                respawn_layout_id,
                            } => {
                                let mut network = network.lock();

                                let mut data = data.lock();
                                if let Some(instance) = data.instances.get_mut(*instance_index) {
                                    network.send_ac_in_range_instance(
                                        instance,
                                        *actor_id,
                                        ActorControlCategory::DeadFadeOut,
                                    );

                                    instance.insert_task(
                                        ClientId::default(),
                                        ObjectId::default(),
                                        DEAD_DESPAWN_TIME,
                                        QueuedTaskData::DeadDespawn {
                                            actor_id: *actor_id,
                                            respawn_layout_id: *respawn_layout_id,
                                        },
                                    );
                                } else {
                                    tracing::warn!(
                                        "Could not find instance to dead fade out {actor_id}?!"
                                    );
                                }
                            }
                            QueuedTaskData::SendDefeated { actor_id, killer } => {
                                let mut network = network.lock();
                                let mut data = data.lock();
                                if let Some(instance) = data.find_actor_instance_mut(*actor_id) {
                                    network.send_ac_in_range_inclusive_instance(
                                        instance,
                                        *actor_id,
                                        ActorControlCategory::Defeated {
                                            id1: *killer,
                                            id2: *killer,
                                        },
                                    );
                                }
                            }
                            QueuedTaskData::DeadDespawn {
                                actor_id,
                                respawn_layout_id,
                            } => {
                                // despawn
                                let mut data = data.lock();
                                if let Some(instance) = data.find_actor_instance_mut(*actor_id) {
                                    let respawn_time = instance
                                        .find_actor(*actor_id)
                                        .map(|actor| {
                                            mob_respawn_time(actor.get_common_spawn().level)
                                        })
                                        .unwrap_or(MOB_RESPAWN_TIME);
                                    let mut network = network.lock();
                                    network.remove_actor(instance, *actor_id);

                                    if let Some(layout_id) = respawn_layout_id {
                                        instance.insert_task(
                                            ClientId::default(),
                                            ObjectId::default(),
                                            respawn_time,
                                            QueuedTaskData::RespawnMob {
                                                layout_id: *layout_id,
                                            },
                                        );
                                    }
                                }
                            }
                            QueuedTaskData::CastEventAction { target } => {
                                let mut data = data.lock();
                                if let Some(instance) =
                                    data.find_actor_instance_mut(task.from_actor_id)
                                    && let Some(director) = &mut instance.directors.first_mut()
                                {
                                    director.event_action_cast(task.from_actor_id, *target);
                                }
                            }
                            QueuedTaskData::QuestEventAction => {
                                let mut network = network.lock();
                                network.send_to(
                                    task.from_id,
                                    FromServer::QuestEventActionCast,
                                    DestinationNetwork::ZoneClients,
                                );
                            }
                            QueuedTaskData::FishBite => {
                                let mut network = network.lock();
                                network.send_to(
                                    task.from_id,
                                    FromServer::FishBite,
                                    DestinationNetwork::ZoneClients,
                                );
                            }
                            QueuedTaskData::SealBossWall { id, place_name } => {
                                let mut data = data.lock();
                                if let Some(instance) =
                                    data.find_actor_instance_mut(task.from_actor_id)
                                    && let Some(director) = &mut instance.directors.first_mut()
                                {
                                    director.seal_boss_wall(*id, *place_name);
                                }
                            }
                            QueuedTaskData::PacketSegment { segment } => {
                                let mut network = network.lock();
                                network.send_to(
                                    task.from_id,
                                    FromServer::PacketSegment(segment.clone(), task.from_actor_id),
                                    DestinationNetwork::ZoneClients,
                                );
                            }
                            QueuedTaskData::EffectResultLands { segment } => {
                                let mut network = network.lock();
                                let data = data.lock();
                                if let Some(instance) = data.find_actor_instance(task.from_actor_id)
                                {
                                    network.send_in_range_inclusive_instance(
                                        task.from_actor_id,
                                        instance,
                                        FromServer::PacketSegment(
                                            segment.clone(),
                                            task.from_actor_id,
                                        ),
                                        DestinationNetwork::ZoneClients,
                                    );
                                }
                            }
                            QueuedTaskData::WarpToPopRange { id } => {
                                let mut data = data.lock();
                                let mut network = network.lock();
                                let mut game_data = game_data.lock();
                                let lua = lua.lock();

                                let from_id = network.find_by_actor(task.from_actor_id).unwrap();

                                change_zone_warp_to_pop_range(
                                    &mut data,
                                    &mut network,
                                    &mut game_data,
                                    &lua,
                                    None, // Means we don't want to change their current instance
                                    *id,
                                    task.from_actor_id,
                                    from_id,
                                    WarpType::Normal,
                                    0,
                                );
                            }
                            QueuedTaskData::ResetCombo => {
                                let mut data = data.lock();
                                if let Some(instance) =
                                    data.find_actor_instance_mut(task.from_actor_id)
                                    && let Some(NetworkedActor::Player {
                                        last_combo_action,
                                        combo_sequence,
                                        ..
                                    }) = instance.find_actor_mut(task.from_actor_id)
                                {
                                    *last_combo_action = 0;
                                    *combo_sequence = 0;
                                }
                            }
                            QueuedTaskData::SheatheWeapon => {
                                let mut data = data.lock();
                                if let Some(instance) = data.instances.get_mut(*instance_index)
                                    && let Some(NetworkedActor::Npc { state, .. }) =
                                        instance.find_actor(task.from_actor_id)
                                    && *state != NpcState::Hate
                                    && *state != NpcState::Dead
                                {
                                    let mut network = network.lock();
                                    network.send_ac_in_range_instance(
                                        instance,
                                        task.from_actor_id,
                                        ActorControlCategory::ToggleWeapon {
                                            shown: false,
                                            unk_flag: 1,
                                            unk2: 1,
                                        },
                                    );
                                    network.send_ac_in_range_instance(
                                        instance,
                                        task.from_actor_id,
                                        ActorControlCategory::SetBattle { battle: false },
                                    );
                                }
                            }
                            QueuedTaskData::RespawnMob { layout_id } => {
                                let mut data = data.lock();
                                if let Some(instance) = data.instances.get_mut(*instance_index)
                                    && let Some(npc) = instance.zone.get_battle_npc(*layout_id)
                                {
                                    let actor_id = ObjectId(fastrand::u32(..));
                                    let config = get_config();
                                    let mut game_data = game_data.lock();
                                    instance.insert_npc(actor_id, npc, &config, &mut game_data);
                                }
                            }
                            QueuedTaskData::EndFate { fate_id } => {
                                let mut data = data.lock();
                                let mut ended_ac = None;
                                let mut unk10_ac = None;
                                let mut rewarded = Vec::new();
                                if let Some(instance) = data.instances.get_mut(*instance_index) {
                                    let participants = instance
                                        .fate_participants
                                        .remove(fate_id)
                                        .unwrap_or_default();
                                    if let Some(fate_instance) =
                                        instance.fates.iter_mut().find(|x| x.fate_id == *fate_id)
                                    {
                                        // Completed (progress 100): everyone who helped gets its rewards.
                                        if fate_instance.progress >= 100 {
                                            rewarded = participants;
                                        }
                                        ended_fate(fate_instance);
                                        ended_ac = Some(fate_instance.create_fate_init_ac());

                                        unk10_fate(fate_instance);
                                        unk10_ac = Some(fate_instance.create_fate_init_ac());
                                    }

                                    // Remove the instance
                                    instance.fates.retain(|x| x.fate_id != *fate_id);
                                }

                                if let Some(instance) = data.instances.get(*instance_index)
                                    && let Some(ended_ac) = ended_ac
                                    && let Some(unk10_ac) = unk10_ac
                                {
                                    let mut network = network.lock();
                                    network.send_to_instance(
                                        ObjectId::default(),
                                        instance,
                                        FromServer::ActorControlSelf(ended_ac),
                                        DestinationNetwork::ZoneClients,
                                    );
                                    network.send_to_instance(
                                        ObjectId::default(),
                                        instance,
                                        FromServer::ActorControlSelf(unk10_ac),
                                        DestinationNetwork::ZoneClients,
                                    );
                                    network.send_to_instance(
                                        ObjectId::default(),
                                        instance,
                                        FromServer::FATEComplete(*fate_id),
                                        DestinationNetwork::ZoneClients,
                                    );
                                    for player in rewarded {
                                        // Only those still here.
                                        if instance.find_actor(player).is_none() {
                                            continue;
                                        }
                                        network.send_to_by_actor_id(
                                            player,
                                            FromServer::FateReward(*fate_id),
                                            DestinationNetwork::ZoneClients,
                                        );
                                    }
                                }

                                // Fill in the FATE slot
                                if let Some(instance) = data.instances.get_mut(*instance_index) {
                                    let mut network = network.lock();
                                    let mut game_data = game_data.lock();
                                    let lua = lua.lock();
                                    start_next_fate(&mut network, &mut game_data, &lua, instance);
                                }
                            }
                            QueuedTaskData::TerminateDirector { handler_id } => {
                                let mut data = data.lock();
                                if let Some(instance) = data.instances.get_mut(*instance_index) {
                                    instance.directors.retain(|x| x.id != *handler_id);

                                    let mut network = network.lock();
                                    network.send_to_instance(
                                        ObjectId::default(),
                                        instance,
                                        FromServer::ActorControlSelf(
                                            ActorControlCategory::TerminateDirector {
                                                handler_id: *handler_id,
                                            },
                                        ),
                                        DestinationNetwork::ZoneClients,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    while let Some(msg) = recv.recv().await {
        let mut to_remove = Vec::new();

        let mut handled = handle_chat_messages(
            data.clone(),
            network.clone(),
            game_data.clone(),
            lua.clone(),
            &msg,
        );
        handled |= handle_social_messages(data.clone(), network.clone(), &msg);
        handled |= handle_zone_messages(
            data.clone(),
            network.clone(),
            game_data.clone(),
            lua.clone(),
            &msg,
        );
        handled |= handle_action_messages(data.clone(), game_data.clone(), network.clone(), &msg);
        handled |= handle_effect_messages(data.clone(), network.clone(), lua.clone(), &msg);
        handled |= handle_director_messages(data.clone(), network.clone(), game_data.clone(), &msg);
        handled |= handle_party_messages(data.clone(), network.clone(), &msg);
        handled |= handle_linkshell_messages(network.clone(), &msg);
        handled |=
            loot::handle_loot_messages(data.clone(), network.clone(), game_data.clone(), &msg);
        handled |= companion::handle_companion_messages(
            data.clone(),
            network.clone(),
            game_data.clone(),
            &msg,
        );
        handled |= party_bot::handle_bot_party_messages(
            data.clone(),
            network.clone(),
            game_data.clone(),
            lua.clone(),
            &msg,
        );

        if !handled {
            match msg {
                ToServer::NewClient(handle) => {
                    tracing::info!(
                        "New zone client {:?} created with actor {}",
                        handle.id,
                        handle.actor_id
                    );

                    let mut network = network.lock();
                    let mut party_id = None;
                    let handle_id = handle.id;

                    // Refresh the party member's client id, if applicable.
                    'outer: for (id, party) in &mut network.parties {
                        for member in &mut party.members {
                            if member.actor_id == handle.actor_id {
                                member.zone_client_id = handle.id;
                                party_id = Some(*id);
                                break 'outer;
                            }
                        }
                    }

                    network
                        .clients
                        .insert(handle.id, (handle.clone(), ClientState::default()));

                    if let Some(party_id) = party_id {
                        network.send_to(
                            handle_id,
                            FromServer::RejoinPartyAfterDisconnect(party_id),
                            DestinationNetwork::ZoneClients,
                        );
                    }
                }
                ToServer::NewChatClient(handle) => {
                    tracing::info!(
                        "New chat client {:?} created with actor {}",
                        handle.id,
                        handle.actor_id
                    );

                    let mut network = network.lock();

                    // Refresh the party member's client id, if applicable.
                    'outer: for party in &mut network.parties.values_mut() {
                        for member in &mut party.members {
                            if member.actor_id == handle.actor_id {
                                member.chat_client_id = handle.id; // The chat connection doesn't get informed here since it'll happen later.
                                break 'outer;
                            }
                        }
                    }

                    network
                        .chat_clients
                        .insert(handle.id, (handle, ClientState::default()));
                }
                ToServer::ReadySpawnPlayer(
                    from_id,
                    from_actor_id,
                    zone_id,
                    position,
                    rotation,
                    city_state_opening,
                ) => {
                    tracing::info!("Player {from_id:?} is spawning into zone {zone_id}....");

                    let mut network = network.lock();
                    let mut data = data.lock();

                    // create a new instance if necessary
                    let instance;
                    {
                        let mut game_data = game_data.lock();
                        let lua = lua.lock();
                        instance = data.ensure_exists(zone_id, &mut game_data, &lua);
                    }

                    instance.insert_empty_actor(from_actor_id);

                    let exit_position;
                    let exit_rotation;
                    if let Some(city_state) = city_state_opening {
                        // If spawning for the initial opening, we need to spawn them at this pop range *as soon as possible*
                        // The reason being is that this helps loading times and the initial camera rotation.
                        // Doing it in the opening Lua script happens far too late, as EnterTerritoryEvent will only be fired after ZoneInit is sent.
                        if let Some((object, _)) = instance
                            .zone
                            .find_pop_range(determine_initial_pop_range(city_state))
                        {
                            let (_, rotation, translation) =
                                Affine3A::from(object.transform).to_scale_rotation_translation();
                            exit_position = Position(translation.into());
                            exit_rotation = euler_to_direction(rotation.to_euler(EulerRot::XYZ));
                        } else {
                            exit_position = position;
                            exit_rotation = rotation;
                        }
                    } else {
                        exit_position = position;
                        exit_rotation = rotation;
                    }

                    // tell the client to load into the zone
                    let msg = FromServer::ChangeZone(
                        zone_id,
                        instance.content_finder_condition_id,
                        instance.weather_id,
                        exit_position,
                        exit_rotation,
                        instance.zone.to_lua_zone(instance.weather_id),
                        instance.content_handler_id(),
                    );

                    network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
                }
                ToServer::ActorMoved(
                    actor_id,
                    position,
                    rotation,
                    anim_type,
                    anim_state,
                    jump_state,
                    party_id,
                ) => {
                    let mut data = data.lock();

                    if let Some(instance) = data.find_actor_instance_mut(actor_id) {
                        let mut moved = false;
                        if let Some((_, spawn)) = instance
                            .actors
                            .iter_mut()
                            .find(|actor| *actor.0 == actor_id)
                        {
                            let common = spawn.get_common_spawn_mut();
                            moved = common.position != position;
                            common.position = position;
                            common.rotation = rotation;
                        }

                        // Send actor move!
                        {
                            let mut network = network.lock();
                            let msg = FromServer::ActorMove(
                                actor_id, position, rotation, anim_type, anim_state, jump_state,
                                None,
                            );
                            network.send_in_range_instance(
                                actor_id,
                                instance,
                                msg,
                                DestinationNetwork::ZoneClients,
                            );
                        }

                        if moved {
                            // Check if the actor has any in-progress actions, and cancel them if so. Moving in the last
                            // half second of a cast doesn't interrupt it ("slidecasting"), as on retail, where a
                            // player's cast resolves half a second before its bar ends.
                            let slidecast_from =
                                Instant::now() + Duration::from_millis(SLIDECAST_WINDOW_MS);
                            for task in instance.find_tasks(actor_id) {
                                if let QueuedTaskData::CastAction { interruptible, .. } = &task.data
                                    && *interruptible
                                    && task.point > slidecast_from
                                {
                                    instance.cancel_task(network.clone(), &task);
                                }
                            }

                            // If the actor moved, and they're in a party, we need to update our information.
                            if let Some(party_id) = party_id {
                                let mut network = network.lock();
                                update_party_position(
                                    &mut network,
                                    &mut data,
                                    party_id,
                                    actor_id,
                                    position,
                                );
                            }
                        }
                    }
                }
                ToServer::ClientTrigger(from_id, from_actor_id, trigger) => {
                    match &trigger.trigger {
                        ClientTriggerCommand::TeleportQuery { aetheryte_id, .. } => {
                            let msg =
                                FromServer::ActorControlSelf(ActorControlCategory::TeleportStart {
                                    insufficient_gil: 0,
                                    aetheryte_id: *aetheryte_id,
                                });

                            {
                                let mut network = network.lock();
                                network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
                                let msg =
                                    FromServer::CheckTeleportSharingEligibility(*aetheryte_id);
                                network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
                            }

                            let mut data = data.lock();
                            if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                && let Some(actor) = instance.find_actor_mut(from_actor_id)
                            {
                                match actor {
                                    NetworkedActor::Player { teleport_query, .. } => {
                                        teleport_query.aetheryte_id = *aetheryte_id as u16
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                        ClientTriggerCommand::WalkInTriggerFinished { .. } => {
                            // This is where we finally release the client after the walk-in trigger.
                            let msg = FromServer::Conditions(Conditions::default());

                            let mut network = network.lock();
                            network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
                        }
                        ClientTriggerCommand::SummonMinion { minion_id } => {
                            let msg = FromServer::ActorSummonsMinion(*minion_id);

                            let mut network = network.lock();
                            network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
                        }
                        ClientTriggerCommand::DespawnMinion => {
                            let msg = FromServer::ActorDespawnsMinion;

                            let mut network = network.lock();
                            network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
                        }
                        ClientTriggerCommand::SetTarget {
                            actor_id,
                            actor_type,
                        } => {
                            // For whatever reason these don't match what the server has to send back, so they cannot be directly reused.
                            let actor_type = match *actor_type {
                                0 => ObjectTypeKind::None,
                                1 => ObjectTypeKind::EObjOrNpc,
                                2 => ObjectTypeKind::Minion,
                                _ => {
                                    tracing::warn!(
                                        "SetTarget: Unknown actor target type {}! Defaulting to None!",
                                        *actor_type
                                    );
                                    ObjectTypeKind::None
                                }
                            };

                            let object_type_id = ObjectTypeId {
                                object_id: *actor_id,
                                object_type: actor_type,
                            };

                            let mut data = data.lock();
                            if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                && let Some(actor) = instance.find_actor_mut(from_actor_id)
                            {
                                actor.get_common_spawn_mut().target_id = object_type_id;
                            }

                            let msg = FromServer::ActorControlTarget(
                                from_actor_id,
                                object_type_id,
                                ActorControlCategory::SetTarget,
                            );

                            let mut network = network.lock();
                            network.send_in_range(
                                from_actor_id,
                                &data,
                                msg,
                                DestinationNetwork::ZoneClients,
                            );
                        }
                        ClientTriggerCommand::SetSoftTarget => {
                            if let ClientTriggerTarget::Actor(actor) = trigger.target {
                                let msg = FromServer::ActorControlTarget(
                                    from_actor_id,
                                    actor,
                                    ActorControlCategory::SetSoftTarget,
                                );

                                let mut network = network.lock();
                                let data = data.lock();
                                network.send_in_range(
                                    from_actor_id,
                                    &data,
                                    msg,
                                    DestinationNetwork::ZoneClients,
                                );
                            }
                        }
                        ClientTriggerCommand::ChangePose { unk1, pose } => {
                            let msg = FromServer::ActorControl(
                                from_actor_id,
                                ActorControlCategory::Pose {
                                    unk1: *unk1,
                                    pose: *pose,
                                },
                            );

                            let mut network = network.lock();
                            let mut data = data.lock();
                            network.send_in_range(
                                from_actor_id,
                                &data,
                                msg,
                                DestinationNetwork::ZoneClients,
                            );

                            // Update data for respawns
                            {
                                if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                    && let Some(actor) = instance.find_actor_mut(from_actor_id)
                                    && let NetworkedActor::Player { spawn, .. } = actor
                                {
                                    spawn.pose = *pose as u8;
                                }
                            }
                        }
                        ClientTriggerCommand::ReapplyPose { unk1, pose } => {
                            let msg = FromServer::ActorControl(
                                from_actor_id,
                                ActorControlCategory::Pose {
                                    unk1: *unk1,
                                    pose: *pose,
                                },
                            );

                            let mut network = network.lock();
                            let mut data = data.lock();
                            network.send_in_range(
                                from_actor_id,
                                &data,
                                msg,
                                DestinationNetwork::ZoneClients,
                            );

                            // Update data for respawns
                            {
                                if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                    && let Some(actor) = instance.find_actor_mut(from_actor_id)
                                    && let NetworkedActor::Player { spawn, .. } = actor
                                {
                                    spawn.pose = *pose as u8;
                                }
                            }
                        }
                        ClientTriggerCommand::Emote { emote, hide_text } => {
                            if let ClientTriggerTarget::Actor(actor) = trigger.target {
                                let msg = FromServer::ActorControlTarget(
                                    from_actor_id,
                                    actor,
                                    ActorControlCategory::Emote {
                                        emote: *emote,
                                        hide_text: *hide_text,
                                    },
                                );

                                let mut network = network.lock();
                                let mut data = data.lock();
                                network.send_in_range(
                                    from_actor_id,
                                    &data,
                                    msg,
                                    DestinationNetwork::ZoneClients,
                                );
                                // A quest may be waiting for it (retail's EventStart of type Emote follows the emote.)
                                network.send_to(
                                    from_id,
                                    FromServer::EmotedAt(*emote, actor),
                                    DestinationNetwork::ZoneClients,
                                );

                                // setup persistence if looping
                                let emote_mode;
                                {
                                    let mut game_data = game_data.lock();
                                    emote_mode = game_data.get_emote_mode(*emote);
                                }

                                if let Some(mode) = emote_mode
                                    && let Some(instance) =
                                        data.find_actor_instance_mut(from_actor_id)
                                {
                                    set_character_mode(
                                        instance,
                                        &mut network,
                                        from_actor_id,
                                        CharacterMode::EmoteLoop,
                                        mode,
                                        true,
                                    );
                                }
                            }
                        }
                        ClientTriggerCommand::ToggleWeapon { shown, unk_flag } => {
                            // Sheathing your weapon also stops auto-attacking, like Sapphire does. The client doesn't
                            // always send its own request to stop, so otherwise the next auto-attack draws it again.
                            if !*shown {
                                let mut data = data.lock();
                                if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                    && let Some(NetworkedActor::Player {
                                        autoattack_target, ..
                                    }) = instance.find_actor_mut(from_actor_id)
                                {
                                    *autoattack_target = None;
                                }
                            }

                            let msg = FromServer::ActorControl(
                                from_actor_id,
                                ActorControlCategory::ToggleWeapon {
                                    shown: *shown,
                                    unk_flag: *unk_flag,
                                    unk2: 1,
                                },
                            );

                            let mut network = network.lock();
                            let data = data.lock();
                            network.send_in_range(
                                from_actor_id,
                                &data,
                                msg,
                                DestinationNetwork::ZoneClients,
                            );
                        }
                        ClientTriggerCommand::ManuallyRemoveEffect {
                            effect_id,
                            source_actor_id,
                            effect_param,
                        } => {
                            // If there is a scheduled task to remove it, cancel it!
                            // This is harmless to keep, but it's better not to clog the queue.
                            {
                                let mut data = data.lock();
                                if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                {
                                    for task in instance.find_tasks(from_actor_id) {
                                        let target_effect_id = *effect_id as u16;
                                        let target_actor_id = *source_actor_id;
                                        // NOTE: I intentionally don't match on effect_param, I don't think that's truly reflective from CT?
                                        if let QueuedTaskData::LoseStatusEffect {
                                            effect_id,
                                            effect_source_actor_id,
                                            ..
                                        } = task.data
                                            && effect_id == target_effect_id
                                            && effect_source_actor_id == target_actor_id
                                        {
                                            instance.cancel_task(network.clone(), &task);
                                        }
                                    }
                                }
                            }

                            remove_effect(
                                network.clone(),
                                data.clone(),
                                lua.clone(),
                                from_id,
                                from_actor_id,
                                *effect_id as u16,
                                *effect_param as u16,
                                *source_actor_id,
                            );
                        }
                        ClientTriggerCommand::SetDistanceRange { range } => {
                            let mut data = data.lock();
                            if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                && let Some(actor) = instance.find_actor_mut(from_actor_id)
                            {
                                match actor {
                                    NetworkedActor::Player { distance_range, .. } => {
                                        *distance_range = *range;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                        ClientTriggerCommand::GimmickJumpLanded { .. } => {
                            let mut data = data.lock();
                            if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                && let Some(actor) = instance.find_actor_mut(from_actor_id)
                            {
                                match actor {
                                    NetworkedActor::Player {
                                        executing_gimmick_jump,
                                        ..
                                    } => {
                                        *executing_gimmick_jump = false;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                        ClientTriggerCommand::SetTitle { title_id } => {
                            let mut data = data.lock();
                            if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                && let Some(actor) = instance.find_actor_mut(from_actor_id)
                            {
                                match actor {
                                    NetworkedActor::Player { spawn, .. } => {
                                        spawn.title_id = *title_id as u16;
                                    }
                                    _ => unreachable!(),
                                }
                            }

                            // inform other players
                            let msg = FromServer::ActorControl(
                                from_actor_id,
                                ActorControlCategory::SetTitle {
                                    title_id: *title_id,
                                },
                            );

                            let mut network = network.lock();
                            network.send_in_range(
                                from_actor_id,
                                &data,
                                msg,
                                DestinationNetwork::ZoneClients,
                            );
                        }
                        ClientTriggerCommand::PlaceWaymark { id, pos } => {
                            let mut network = network.lock();
                            let data = data.lock();

                            update_party_waymark(
                                &mut network,
                                &data,
                                from_actor_id,
                                *id,
                                Some(*pos),
                            );
                        }
                        ClientTriggerCommand::ClearWaymark { id } => {
                            let mut network = network.lock();
                            let data = data.lock();

                            update_party_waymark(&mut network, &data, from_actor_id, *id, None);
                        }
                        ClientTriggerCommand::ClearAllWaymarks => {
                            let mut network = network.lock();
                            let data = data.lock();

                            // Clearing all waymarks is equivalent to sending a completely blank preset.
                            update_party_waymarks(
                                &mut network,
                                &data,
                                from_actor_id,
                                WaymarkPreset::default(),
                            );
                        }
                        ClientTriggerCommand::ToggleSign { sign_id, .. } => {
                            let ClientTriggerTarget::Actor(target_actor) = trigger.target else {
                                continue;
                            };

                            let mut network = network.lock();
                            let msg = FromServer::TargetSignToggled(
                                *sign_id,
                                from_actor_id,
                                target_actor,
                            );

                            network.send_to_party_or_self(from_actor_id, msg);

                            // If we're in a party, keep track of the actor that was just marked.
                            if let Some(party_id) =
                                get_party_id_from_actor_id(&network, from_actor_id)
                            {
                                let party = network.parties.get_mut(&party_id).unwrap();
                                let sign_id = *sign_id as usize;
                                if sign_id < NUM_TARGET_SIGNS {
                                    party.target_signs[sign_id] = target_actor;
                                } else {
                                    tracing::error!(
                                        "Client tried to assign target sign id {sign_id}, but there are only {NUM_TARGET_SIGNS} currently known. Update the constant if necessary!"
                                    );
                                }
                            }
                        }
                        ClientTriggerCommand::RequestDuel { actor_id } => {
                            let mut network = network.lock();
                            network.send_to_by_actor_id(
                                from_actor_id,
                                FromServer::ActorControlSelf(ActorControlCategory::SetPvPState {
                                    state: 2,
                                }),
                                DestinationNetwork::ZoneClients,
                            );

                            let account_id;
                            {
                                let Some((handle, _)) = network.get_by_actor_mut(from_actor_id)
                                else {
                                    continue;
                                };
                                account_id = handle.account_id;
                            }

                            let opponent_content_id;
                            let opponent_object_id;
                            let opponent_name;
                            {
                                let Some((handle, _)) = network.get_by_actor_mut(*actor_id) else {
                                    continue;
                                };
                                opponent_content_id = handle.content_id;
                                opponent_object_id = *actor_id;

                                let data = data.lock();
                                let Some(instance) = data.find_actor_instance(*actor_id) else {
                                    continue;
                                };

                                let Some(actor) = instance.find_actor(*actor_id) else {
                                    continue;
                                };

                                opponent_name = actor.get_common_spawn().name.clone();
                            }

                            let config = get_config();

                            let ipc =
                                ServerZoneIpcSegment::new(ServerZoneIpcData::DuelInformation {
                                    account_id,
                                    opponent_content_id,
                                    opponent_object_id,
                                    world_id: config.world.world_id,
                                    unk1: 7957,
                                    unk2: 1,
                                    opponent_name,
                                });
                            network.send_to_by_actor_id(
                                from_actor_id,
                                FromServer::PacketSegment(ipc, from_actor_id),
                                DestinationNetwork::ZoneClients,
                            );

                            let mut data = data.lock();
                            let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                                continue;
                            };

                            // Update our player
                            {
                                let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                                    continue;
                                };

                                match actor {
                                    NetworkedActor::Player {
                                        dueling_opponent_id,
                                        ..
                                    } => *dueling_opponent_id = *actor_id,
                                    _ => unreachable!(),
                                };
                            }

                            // Update the opponent
                            {
                                let Some(actor) = instance.find_actor_mut(*actor_id) else {
                                    continue;
                                };

                                match actor {
                                    NetworkedActor::Player {
                                        dueling_opponent_id,
                                        ..
                                    } => *dueling_opponent_id = from_actor_id,
                                    _ => unreachable!(),
                                };
                            }
                        }
                        ClientTriggerCommand::RequestDuelResponse { cancel } => {
                            if *cancel {
                                let mut data = data.lock();
                                let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                else {
                                    continue;
                                };

                                let other_actor_id;

                                // Update our player
                                {
                                    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                                        continue;
                                    };

                                    match actor {
                                        NetworkedActor::Player {
                                            dueling_opponent_id,
                                            ..
                                        } => {
                                            other_actor_id = *dueling_opponent_id;
                                            *dueling_opponent_id = ObjectId::default();
                                        }
                                        _ => unreachable!(),
                                    };
                                }

                                // Update the opponent
                                {
                                    let Some(actor) = instance.find_actor_mut(other_actor_id)
                                    else {
                                        continue;
                                    };

                                    match actor {
                                        NetworkedActor::Player {
                                            dueling_opponent_id,
                                            ..
                                        } => *dueling_opponent_id = ObjectId::default(),
                                        _ => unreachable!(),
                                    };
                                }
                            } else {
                                // If not cancelling, then we need to send a confirmation to the opponent...
                                let mut network = network.lock();

                                let data = data.lock();
                                let Some(instance) = data.find_actor_instance(from_actor_id) else {
                                    continue;
                                };

                                let Some(actor) = instance.find_actor(from_actor_id) else {
                                    continue;
                                };

                                let dueling_opponent_id = match actor {
                                    NetworkedActor::Player {
                                        dueling_opponent_id,
                                        ..
                                    } => dueling_opponent_id,
                                    _ => unreachable!(),
                                };

                                let account_id;
                                {
                                    let Some((handle, _)) =
                                        network.get_by_actor_mut(*dueling_opponent_id)
                                    else {
                                        continue;
                                    };
                                    account_id = handle.account_id;
                                }

                                let opponent_content_id;
                                let opponent_object_id;
                                let opponent_name;
                                {
                                    let Some((handle, _)) = network.get_by_actor_mut(from_actor_id)
                                    else {
                                        continue;
                                    };
                                    opponent_content_id = handle.content_id;
                                    opponent_object_id = from_actor_id;

                                    opponent_name = actor.get_common_spawn().name.clone();
                                }

                                let config = get_config();

                                let ipc =
                                    ServerZoneIpcSegment::new(ServerZoneIpcData::DuelInformation {
                                        account_id,
                                        opponent_content_id,
                                        opponent_object_id,
                                        world_id: config.world.world_id,
                                        unk1: 7957,
                                        unk2: 0,
                                        opponent_name,
                                    });
                                network.send_to_by_actor_id(
                                    *dueling_opponent_id,
                                    FromServer::ActorControlSelf(
                                        ActorControlCategory::SetPvPState { state: 3 },
                                    ),
                                    DestinationNetwork::ZoneClients,
                                );
                                network.send_to_by_actor_id(
                                    *dueling_opponent_id,
                                    FromServer::ActorControlSelf(
                                        ActorControlCategory::SetPvPState { state: 4 },
                                    ),
                                    DestinationNetwork::ZoneClients,
                                );
                                network.send_to_by_actor_id(
                                    *dueling_opponent_id,
                                    FromServer::PacketSegment(ipc, *dueling_opponent_id),
                                    DestinationNetwork::ZoneClients,
                                );
                            }
                        }
                        ClientTriggerCommand::DuelDecision { decline } => {
                            // TODO: what happens if they do decline?
                            if !*decline {
                                let data = data.lock();
                                let Some(instance) = data.find_actor_instance(from_actor_id) else {
                                    continue;
                                };

                                let Some(actor) = instance.find_actor(from_actor_id) else {
                                    continue;
                                };

                                let dueling_opponent_id = match actor {
                                    NetworkedActor::Player {
                                        dueling_opponent_id,
                                        ..
                                    } => dueling_opponent_id,
                                    _ => unreachable!(),
                                };

                                tracing::info!(
                                    "Duel has begun between {from_actor_id} and {dueling_opponent_id}!"
                                );

                                let mut network = network.lock();
                                // unknown
                                network.send_ac_in_range_inclusive(
                                    &data,
                                    *dueling_opponent_id,
                                    ActorControlCategory::SetPvPState { state: 5 },
                                );
                                network.send_ac_in_range_inclusive(
                                    &data,
                                    from_actor_id,
                                    ActorControlCategory::SetPvPState { state: 5 },
                                );

                                // unknown ver. 2
                                network.send_ac_in_range_inclusive(
                                    &data,
                                    *dueling_opponent_id,
                                    ActorControlCategory::SetPvPState { state: 6 },
                                );
                                network.send_ac_in_range_inclusive(
                                    &data,
                                    from_actor_id,
                                    ActorControlCategory::SetPvPState { state: 6 },
                                );

                                // begin countdown
                                network.send_to_by_actor_id(
                                    *dueling_opponent_id,
                                    FromServer::ActorControlSelf(
                                        ActorControlCategory::StartDuelCountdown {
                                            opponent_id: from_actor_id,
                                        },
                                    ),
                                    DestinationNetwork::ZoneClients,
                                );
                                network.send_to_by_actor_id(
                                    from_actor_id,
                                    FromServer::ActorControlSelf(
                                        ActorControlCategory::StartDuelCountdown {
                                            opponent_id: *dueling_opponent_id,
                                        },
                                    ),
                                    DestinationNetwork::ZoneClients,
                                );

                                // BATTLE
                                network.send_to_by_actor_id(
                                    *dueling_opponent_id,
                                    FromServer::ActorControlSelf(ActorControlCategory::SetBattle {
                                        battle: true,
                                    }),
                                    DestinationNetwork::ZoneClients,
                                );
                                network.send_to_by_actor_id(
                                    from_actor_id,
                                    FromServer::ActorControlSelf(ActorControlCategory::SetBattle {
                                        battle: true,
                                    }),
                                    DestinationNetwork::ZoneClients,
                                );
                            }
                        }
                        ClientTriggerCommand::ResetStrikingDummy { id } => {
                            let mut data = data.lock();
                            let Some(instance) = data.find_actor_instance_mut(*id) else {
                                continue;
                            };

                            let Some(actor) = instance.find_actor_mut(*id) else {
                                continue;
                            };

                            let NetworkedActor::Npc {
                                state,
                                navmesh_target: current_target,
                                enmity,
                                spawn,
                                ..
                            } = actor
                            else {
                                continue;
                            };

                            // Forget everyone who attacked it, otherwise it goes straight back to whoever it hated most.
                            enmity.clear();
                            *state = NpcState::Wander;
                            *current_target = None;
                            spawn.common.target_id = ObjectTypeId::default();

                            // TODO: throw this into a generic de-aggro thing eventually
                            let mut network = network.lock();
                            network.send_ac_in_range(
                                &data,
                                *id,
                                ActorControlCategory::SetBattle { battle: false },
                            );
                        }
                        ClientTriggerCommand::EmoteInterrupted => {
                            let data = data.lock();
                            let mut network = network.lock();
                            network.send_ac_in_range_inclusive(
                                &data,
                                from_actor_id,
                                ActorControlCategory::InterruptEmote,
                            );
                        }
                        ClientTriggerCommand::LoopingEmoteInterrupted => {
                            let mut data = data.lock();
                            let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                                continue;
                            };

                            let mut network = network.lock();
                            set_character_mode(
                                instance,
                                &mut network,
                                from_actor_id,
                                CharacterMode::Normal,
                                0,
                                true,
                            );
                        }
                        ClientTriggerCommand::ToggleAutoAttack { on, target, .. } => {
                            let mut data = data.lock();
                            if let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                && let Some(actor) = instance.find_actor_mut(from_actor_id)
                            {
                                match actor {
                                    NetworkedActor::Player {
                                        autoattack_target, ..
                                    } => {
                                        if *on {
                                            *autoattack_target = Some(*target);
                                        } else {
                                            *autoattack_target = None;
                                        }
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                        ClientTriggerCommand::StartPreparingFATE { fate_id, .. } => {
                            let mut data = data.lock();
                            if let Some(instance) = data.find_actor_instance_mut(from_actor_id) {
                                let mut network = network.lock();
                                start_fate(&mut network, instance, *fate_id);
                            }
                        }
                        ClientTriggerCommand::AcceptRevive { kind } => {
                            let mut data = data.lock();
                            // Accepting a Raise (and one was offered): back up where they lie.
                            if *kind == REVIVE_ACCEPT_RAISE
                                && let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                                && instance
                                    .find_actor(from_actor_id)
                                    .and_then(|actor| actor.status_effects())
                                    .is_some_and(|effects| {
                                        effects.get(STATUS_REVIVE_OFFER).is_some()
                                    })
                            {
                                revive_in_place(network.clone(), instance, from_id, from_actor_id);
                            } else if let Some(instance) =
                                data.find_actor_instance_mut(from_actor_id)
                            {
                                // Otherwise back to the starting point.
                                if let Some(actor) = instance.find_actor_mut(from_actor_id) {
                                    // Full HP and MP, and no Weakness or Brink of Death (retail.)
                                    let common = actor.get_common_spawn_mut();
                                    common.health_points = common.max_health_points;
                                    common.resource_points = common.max_resource_points;
                                    if let Some(effects) = actor.status_effects_mut() {
                                        effects.remove(STATUS_REVIVE_OFFER);
                                        effects.remove(STATUS_WEAKNESS);
                                        effects.remove(STATUS_BRINK_OF_DEATH);
                                    }
                                }
                                update_actor_hp_mp(network.clone(), instance, from_actor_id);

                                let mut network = network.lock();
                                let mut game_data = game_data.lock();

                                let ipc = ServerZoneIpcSegment::new(
                                    ServerZoneIpcData::PrepareZoning(PrepareZoning {
                                        territory_type_id: instance.zone.id,
                                        warp_type: WarpType::Resurrection,
                                        vfx_id: 113,
                                        hide_character: 2,
                                        fade_out_delay: 1,
                                        flags: PrepareZoningFlag::UNK2,
                                        ..Default::default()
                                    }),
                                );

                                network.send_to_by_actor_id(
                                    from_actor_id,
                                    FromServer::PacketSegment(ipc, from_actor_id),
                                    DestinationNetwork::ZoneClients,
                                );

                                change_zone_warp_to_entrance(
                                    &mut network,
                                    &mut game_data,
                                    instance,
                                    false,
                                    from_id,
                                    WarpType::Resurrection,
                                );
                            }
                        }
                        ClientTriggerCommand::DirectorTrigger {
                            handler_id,
                            trigger,
                        } => {
                            handle_director_trigger(
                                network.clone(),
                                game_data.clone(),
                                from_actor_id,
                                *handler_id,
                                trigger,
                            );
                        }
                        ClientTriggerCommand::PetAction { action_id }
                            if *action_id == pet::PET_ACTION_AWAY =>
                        {
                            let mut data = data.lock();
                            if let Some(instance) = data.find_actor_instance_mut(from_actor_id) {
                                let mut network = network.lock();
                                pet::dismiss(&mut network, instance, from_actor_id);
                            }
                        }
                        ClientTriggerCommand::ReadyForDirectorVars => {
                            send_director_vars(
                                data.clone(),
                                network.clone(),
                                game_data.clone(),
                                from_actor_id,
                            );
                        }
                        _ => tracing::warn!("Unknown client trigger {:#?}", trigger),
                    }
                }
                ToServer::Config(_from_id, from_actor_id, config) => {
                    let mut data = data.lock();

                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                        continue;
                    };

                    let NetworkedActor::Player { spawn, .. } = actor else {
                        continue;
                    };

                    // update their stored state so it's correctly sent on new spawns
                    spawn.common.display_flags = config.display_flag.into();

                    let mut network = network.lock();
                    let msg = FromServer::PacketSegment(
                        ServerZoneIpcSegment::new(ServerZoneIpcData::Config(config.clone())),
                        from_actor_id,
                    );

                    network.send_in_range_inclusive_instance(
                        from_actor_id,
                        instance,
                        msg,
                        DestinationNetwork::ZoneClients,
                    );
                }
                ToServer::Equip(
                    from_actor_id,
                    main_weapon_id,
                    sub_weapon_id,
                    model_ids,
                    second_model_stain_ids,
                ) => {
                    let mut data = data.lock();

                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                        continue;
                    };

                    let NetworkedActor::Player { spawn, .. } = actor else {
                        continue;
                    };

                    // update their stored state so it's correctly sent on new spawns
                    spawn.common.main_weapon_model = main_weapon_id;
                    spawn.common.sec_weapon_model = sub_weapon_id;
                    spawn.common.models = model_ids;
                    spawn.common.second_model_stain_ids = second_model_stain_ids;

                    // Inform all clients about their new equipped model ids
                    let msg = FromServer::ActorEquip(
                        from_actor_id,
                        main_weapon_id,
                        sub_weapon_id,
                        model_ids,
                        second_model_stain_ids,
                    );

                    let mut network = network.lock();
                    network.send_in_range_inclusive_instance(
                        from_actor_id,
                        instance,
                        msg,
                        DestinationNetwork::ZoneClients,
                    );
                }
                ToServer::Disconnected(from_id, from_actor_id) => {
                    let mut network = network.lock();
                    network.to_remove.push(from_id);

                    // Tell our sibling chat connection that it's time to go too.
                    network.send_to_by_actor_id(
                        from_actor_id,
                        FromServer::ChatDisconnected,
                        DestinationNetwork::ChatClients,
                    );

                    // Remove them from any relevant linkshells.
                    network
                        .linkshells
                        .iter_mut()
                        .for_each(|(_, linkshell)| linkshell.retain(|m| *m != from_actor_id));
                    // Clean up any empty linkshells.
                    network.linkshells.retain(|_, shell| !shell.is_empty());
                }
                ToServer::ActorSummonsMinion(from_actor_id, minion_id) => {
                    let mut network = network.lock();
                    let mut data = data.lock();

                    set_player_minion(&mut data, &mut network, minion_id, from_actor_id);
                }
                ToServer::ActorDespawnsMinion(from_actor_id) => {
                    let mut network = network.lock();
                    let mut data = data.lock();

                    set_player_minion(&mut data, &mut network, 0, from_actor_id);
                }
                ToServer::ChatDisconnected(from_id) => {
                    let mut network = network.lock();
                    network.to_remove_chat.push(from_id);
                }
                ToServer::JoinContent(
                    from_id,
                    from_actor_id,
                    content_id,
                    settings,
                    duty_support,
                ) => {
                    // For now, just send them to do the zone if they do anything
                    let zone_id;
                    {
                        let mut game_data = game_data.lock();
                        zone_id = game_data.find_zone_for_content(content_id);
                    }

                    let mut actor_ids = Vec::new();

                    // Send all party members to this instanced content
                    let mut network = network.lock();
                    if let Some(party_id) = get_party_id_from_actor_id(&network, from_actor_id) {
                        if let Some(party) = network.parties.get(&party_id) {
                            for member in &party.members {
                                if member.is_valid() && member.is_online() {
                                    actor_ids.push((member.zone_client_id, member.actor_id));
                                }
                            }
                        }
                    } else {
                        actor_ids.push((from_id, from_actor_id));
                    }

                    if let Some(zone_id) = zone_id {
                        let mut data = data.lock();

                        for (_, actor_id) in &actor_ids {
                            remove_actor_from_instance(&mut data, &mut network, *actor_id);
                        }

                        // then find or create a new instance with the zone id and content finder condition
                        let mut game_data = game_data.lock();
                        let lua = lua.lock();
                        if let Some(target_instance) = data.create_instance_for_content(
                            zone_id,
                            content_id,
                            settings,
                            &mut game_data,
                            &lua,
                        ) {
                            // Alone through the Duty Finder: bots fill the party once they've loaded in; through Duty
                            // Support, the members chosen (party NPCs) and bots for anyone not recorded.
                            if actor_ids.len() == 1 {
                                target_instance.pending_duty_bots.insert(actor_ids[0].1);
                                if let Some(members) = duty_support {
                                    target_instance
                                        .pending_duty_support
                                        .insert(actor_ids[0].1, members);
                                }
                            }
                            for (client_id, actor_id) in &actor_ids {
                                target_instance.insert_empty_actor(*actor_id);

                                change_zone_warp_to_entrance(
                                    &mut network,
                                    &mut game_data,
                                    target_instance,
                                    true,
                                    *client_id,
                                    WarpType::Normal,
                                );
                            }
                        } else {
                            tracing::warn!("Failed to create a new instance for content?!");
                        }
                    } else {
                        tracing::warn!("Failed to find zone id for content?!");
                    }
                }
                ToServer::JoinQuestBattle(from_id, from_actor_id, quest_battle_id, zone_id) => {
                    let mut data = data.lock();
                    let mut network = network.lock();

                    remove_actor_from_instance(&mut data, &mut network, from_actor_id);

                    let mut game_data = game_data.lock();
                    let lua = lua.lock();
                    let target_instance = data.create_instance_for_quest_battle(
                        zone_id,
                        quest_battle_id,
                        &mut game_data,
                        &lua,
                    );
                    target_instance.insert_empty_actor(from_actor_id);

                    let start = target_instance.quest_battle_start;
                    if start.is_some() {
                        do_change_zone(
                            &mut network,
                            target_instance,
                            true,
                            start.map(|x| x.0),
                            start.map(|x| x.1),
                            from_id,
                            WarpType::Normal,
                        );
                    } else {
                        change_zone_warp_to_entrance(
                            &mut network,
                            &mut game_data,
                            target_instance,
                            true,
                            from_id,
                            WarpType::Normal,
                        );
                    }
                }
                ToServer::LeaveContent(
                    from_client_id,
                    from_actor_id,
                    old_zone_id,
                    old_position,
                    old_rotation,
                ) => {
                    let mut data = data.lock();
                    let mut network = network.lock();

                    // Their bots stay behind (and go with the instance), and their party list empties.
                    if party_bot::dismiss_party_bots(&mut data, &mut network, from_actor_id) > 0 {
                        network.send_to(
                            from_client_id,
                            FromServer::BotParty(None),
                            DestinationNetwork::ZoneClients,
                        );
                    }

                    remove_actor_from_instance(&mut data, &mut network, from_actor_id);

                    // create a new instance if necessary
                    let instance;
                    {
                        let mut game_data = game_data.lock();
                        let lua = lua.lock();
                        instance = data.ensure_exists(old_zone_id, &mut game_data, &lua);
                    }

                    instance.insert_empty_actor(from_actor_id);

                    // tell the client to load into the zone
                    let msg = FromServer::ChangeZone(
                        old_zone_id,
                        instance.content_finder_condition_id,
                        instance.weather_id,
                        old_position,
                        old_rotation,
                        instance.zone.to_lua_zone(instance.weather_id),
                        instance.content_handler_id(),
                    );
                    network.send_to(from_client_id, msg, DestinationNetwork::ZoneClients);
                }
                ToServer::UpdateConditions(from_actor_id, new_conditions) => {
                    // update their stored state
                    let mut data = data.lock();

                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                        continue;
                    };

                    let NetworkedActor::Player { conditions, .. } = actor else {
                        continue;
                    };

                    *conditions = new_conditions;
                }
                ToServer::Kill(_from_id, from_actor_id) => {
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };
                    kill_actor(network.clone(), instance, from_actor_id)
                }
                ToServer::SetHP(_from_id, from_actor_id, hp) => {
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                        continue;
                    };

                    actor.get_common_spawn_mut().health_points = hp;

                    update_actor_hp_mp(network.clone(), instance, from_actor_id);
                }
                ToServer::SetMP(_from_id, from_actor_id, mp) => {
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                        continue;
                    };

                    actor.get_common_spawn_mut().resource_points = mp;

                    update_actor_hp_mp(network.clone(), instance, from_actor_id);
                }
                ToServer::RestoreResourcePoints(from_actor_id, amount) => {
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };
                    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                        continue;
                    };
                    let common = actor.get_common_spawn_mut();
                    common.resource_points = common
                        .resource_points
                        .saturating_add(amount)
                        .min(common.max_resource_points);
                    update_actor_hp_mp(network.clone(), instance, from_actor_id);
                }
                ToServer::SpawnQuestBattleNpc(_from_id, from_actor_id, layout_id) => {
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    // Only one at a time: searching again while it's still around doesn't bring out another.
                    if instance.find_npc(layout_id).is_some() {
                        continue;
                    }

                    let Some(npc) = instance.zone.get_battle_npc(layout_id) else {
                        tracing::warn!(
                            "A quest tried to spawn battle NPC {layout_id}, but it isn't in this zone's drop-ins!"
                        );
                        continue;
                    };

                    let npc_id = ObjectId(fastrand::u32(..));
                    {
                        let config = get_config();
                        let mut game_data = game_data.lock();
                        instance.insert_npc(npc_id, npc, &config, &mut game_data);
                    }
                    // Quest enemies don't come back as regular monsters once defeated.
                    instance.quest_spawned_npcs.insert(npc_id);

                    // It comes out already after the player who brought it out.
                    if let Some(NetworkedActor::Npc { enmity, .. }) =
                        instance.find_actor_mut(npc_id)
                    {
                        enmity.insert(from_actor_id, 1);
                    }
                }
                ToServer::SpawnRetainer(
                    from_actor_id,
                    retainer_id,
                    spawn,
                    bell_id,
                    player_position,
                ) => {
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };
                    let mut spawn = spawn;
                    // At the bell's spot, facing the player, as on retail; without one, in front of the player.
                    match instance.zone.retainer_spot(bell_id) {
                        Some((pop_range, position, how)) => {
                            tracing::info!(
                                "Retainer spot for bell {bell_id}: pop range {pop_range} ({how}), {:.2} yalms from the player",
                                position.0.distance(player_position.0)
                            );
                            spawn.common.position = position;
                            spawn.common.layout_id = pop_range;
                            spawn.common.rotation =
                                crate::retainer::facing(position, player_position);
                        }
                        None => tracing::info!(
                            "No retainer spot for bell {bell_id} in zone {}: the retainer stands in front of the player",
                            instance.zone.id
                        ),
                    }
                    // Everyone nearby sees them, as on retail; the usual spawning takes care of that.
                    let config = get_config();
                    let mut game_data = game_data.lock();
                    instance.insert_npc(retainer_id, *spawn, &config, &mut game_data);
                }
                ToServer::DespawnRetainer(from_actor_id, retainer_id) => {
                    let mut network = network.lock();
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };
                    if instance.find_actor(retainer_id).is_some() {
                        network.remove_actor(instance, retainer_id);
                    }
                }
                ToServer::SetNewStatValues(from_actor_id, level, class_job, new_parameters) => {
                    // Update internal data model
                    {
                        let mut data = data.lock();
                        let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                            continue;
                        };

                        let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                            continue;
                        };

                        // Job gauges don't carry over between classes, and retail empties them on a level sync too.
                        if (actor.get_common_spawn().class_job != class_job
                            || actor.get_common_spawn().level != level)
                            && let NetworkedActor::Player { job_gauge, .. } = actor
                        {
                            job_gauge.reset();
                        }

                        let common = actor.get_common_spawn_mut();
                        common.level = level;
                        common.max_health_points = new_parameters.hp;
                        common.max_resource_points =
                            crate::common::max_resource_points(class_job, &new_parameters);
                        // Syncing down cuts HP and MP to the new maximum (retail kept them as they were when the sync
                        // ended, rather than filling them up.)
                        common.health_points = common.health_points.min(common.max_health_points);
                        common.resource_points =
                            common.resource_points.min(common.max_resource_points);
                        actor.get_common_spawn_mut().class_job = class_job;

                        if let NetworkedActor::Player { parameters, .. } = actor {
                            *parameters = new_parameters.clone();
                        }

                        // The only way the game can reliably set these stats is via StatusEffectList (REALLY)
                        send_effects_list(network.clone(), instance, from_actor_id);
                    }
                }
                ToServer::QuestEventAction(from_client_id, from_actor_id, cast_time) => {
                    // Finish the cast when the client's progress bar does, unless an enemy's attack interrupts it
                    // first (see interrupt_event_action.)
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };
                    instance.insert_task(
                        from_client_id,
                        from_actor_id,
                        Duration::from_secs(cast_time as u64),
                        QueuedTaskData::QuestEventAction,
                    );
                }
                ToServer::Fish(from_client_id, from_actor_id) => {
                    let mut data = data.lock();
                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    instance.insert_task(
                        from_client_id,
                        from_actor_id,
                        Duration::from_secs(2),
                        QueuedTaskData::FishBite,
                    );
                }
                ToServer::ReloadScripts => {
                    let mut lua = lua.lock();
                    if let Err(err) = lua.init(game_data.clone()) {
                        tracing::warn!("Failed to load Init.lua: {:?}", err);
                    }
                }
                ToServer::Dismounted(from_actor_id, party_id) => {
                    let mut network = network.lock();
                    let data = data.lock();

                    let mut ids_to_dismount = Vec::new();
                    ids_to_dismount.push(from_actor_id);

                    if let Some(party_id) = party_id
                        && let Some(party) = network.parties.get_mut(&party_id)
                    {
                        for member in &mut party.members {
                            // If the dismounting player is this member's driver, this member needs to be dismounted too.
                            if member.pillion_driver_id == from_actor_id {
                                ids_to_dismount.push(member.actor_id);
                            }
                            // If this member is dismounting manually while riding pillion, there is no longer a driver for them.
                            if member.actor_id == from_actor_id
                                && member.pillion_driver_id != ObjectId::default()
                            {
                                member.pillion_driver_id = ObjectId::default();
                            }
                        }
                    }

                    for id in ids_to_dismount {
                        let Some(instance) = data.find_actor_instance(id) else {
                            continue;
                        };

                        let msg = FromServer::ActorDismounted(id);
                        network.send_in_range_inclusive_instance(
                            id,
                            instance,
                            msg,
                            DestinationNetwork::ZoneClients,
                        );
                    }
                }
                ToServer::SetOnlineStatus(from_actor_id, online_status) => {
                    let mut network = network.lock();
                    let data = data.lock();
                    network.send_ac_in_range_inclusive(
                        &data,
                        from_actor_id,
                        ActorControlCategory::SetStatusIcon {
                            icon: online_status,
                        },
                    );
                }
                ToServer::SetCharacterMode(from_actor_id, mode, arg) => {
                    // ACS is sent by the ZoneConnection
                    let mut data = data.lock();
                    let mut network = network.lock();

                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    set_character_mode(instance, &mut network, from_actor_id, mode, arg, false);
                }
                ToServer::BroadcastActorControl(from_actor_id, actor_control) => {
                    let mut network = network.lock();
                    let data = data.lock();
                    network.send_ac_in_range(&data, from_actor_id, actor_control);
                }
                ToServer::RemoveCooldowns(actor_id) => {
                    let mut data = data.lock();

                    let Some(instance) = data.find_actor_instance_mut(actor_id) else {
                        continue;
                    };

                    let Some(actor) = instance.find_actor_mut(actor_id) else {
                        continue;
                    };

                    let NetworkedActor::Player {
                        remove_cooldowns, ..
                    } = actor
                    else {
                        continue;
                    };

                    *remove_cooldowns = true;
                }
                ToServer::Jump(from_id, name) => {
                    let mut data = data.lock();
                    let mut network = network.lock();
                    let mut game_data = game_data.lock();
                    let lua = lua.lock();

                    let to_actor_id = data.find_actor_by_name(&name);

                    change_zone_to_player(
                        &mut network,
                        &mut data,
                        &mut game_data,
                        &lua,
                        from_id,
                        to_actor_id,
                    );
                }
                ToServer::Call(from_actor_id, name) => {
                    let mut data = data.lock();
                    let mut network = network.lock();
                    let mut game_data = game_data.lock();
                    let lua = lua.lock();

                    let actor_id = data.find_actor_by_name(&name);
                    if let Some(client_id) = network.find_by_actor(actor_id) {
                        change_zone_to_player(
                            &mut network,
                            &mut data,
                            &mut game_data,
                            &lua,
                            client_id,
                            from_actor_id,
                        );
                    }
                }
                ToServer::EquipGlasses(from_actor_id, slot, id) => {
                    let mut data = data.lock();

                    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
                        continue;
                    };

                    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                        continue;
                    };

                    let NetworkedActor::Player { spawn, .. } = actor else {
                        continue;
                    };

                    // update their stored state so it's correctly sent on new spawns
                    spawn.common.glasses_ids[slot as usize] = id as u16;

                    let mut network = network.lock();
                    network.send_ac_in_range_inclusive_instance(
                        instance,
                        from_actor_id,
                        ActorControlCategory::SetGlasses { index: slot, id },
                    );
                }
                ToServer::ReloadFestivals => {
                    // TODO: This would override festivals in special zones that use this for content (like Ocean fishing) currently
                    // TODO: what about the latter 4 festivals?
                    let mut network = network.lock();
                    let config = get_config();
                    let clients: Vec<ClientId> = network.clients.keys().copied().collect();
                    for id in clients {
                        network.send_to(
                            id,
                            FromServer::ActorControlSelf(ActorControlCategory::SetFestival {
                                festival1: config.world.active_festivals[0] as u32,
                                festival2: config.world.active_festivals[1] as u32,
                                festival3: config.world.active_festivals[2] as u32,
                                festival4: config.world.active_festivals[3] as u32,
                            }),
                            DestinationNetwork::ZoneClients,
                        );
                    }
                }
                ToServer::SpawnBattleNpc(bnpc_base_id) => {
                    // Currently spawns it on the first player we find
                    let mut data = data.lock();
                    let mut game_data = game_data.lock();
                    let network = network.lock();
                    if let Some(first_client) = network.clients.values().collect::<Vec<_>>().first()
                    {
                        spawn_custom_bnpc(
                            &mut data,
                            &mut game_data,
                            first_client.0.actor_id,
                            bnpc_base_id,
                            405,
                        );
                    }
                }
                ToServer::FatalError(err) => return Err(err),
                _ => {
                    tracing::error!("Received a ToServer message we don't handle yet: {msg:#?}");
                }
            }
        }

        // Remove any clients that errored out
        {
            let mut network = network.lock();

            network.to_remove.append(&mut to_remove);

            for remove_id in std::mem::take(&mut network.to_remove) {
                tracing::info!(
                    "Removing {remove_id:?} from the world server, its connection is gone"
                );

                // remove any actors they had
                let mut actor_id = None;
                for (id, (handle, _)) in &mut network.clients {
                    if *id == remove_id {
                        actor_id = Some(handle.actor_id);
                    }
                }

                if let Some(actor_id) = actor_id {
                    let mut data = data.lock();
                    // remove them from the instance
                    if let Some(current_instance) = data.find_actor_instance_mut(actor_id) {
                        network.remove_actor(current_instance, actor_id);
                    }
                }

                network.clients.remove(&remove_id);
            }

            for remove_id in std::mem::take(&mut network.to_remove_chat) {
                network.chat_clients.remove(&remove_id);
            }
        }

        // Commit parties back to database as necessary
        {
            let mut network = network.lock();

            // This may seem weird, but currently only the database connection exists on the ZoneConnection side. So we hijack the first client to do our dirty work.
            if network.commit_parties && !network.clients.is_empty() {
                let parties = network.parties.clone();
                let client_id = network.clients.keys().copied().collect::<Vec<ClientId>>()[0];
                network.send_to(
                    client_id,
                    FromServer::CommitParties(parties),
                    DestinationNetwork::ZoneClients,
                );
                network.commit_parties = false;
            }
        }
    }
    Ok(())
}
