use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    ClientId, FromServer, GameData, Navmesh, StatusEffects,
    lua::KawariLua,
    server::{
        WorldServer,
        action::cancel_action,
        actor::{NetworkedActor, NpcState},
        director::DirectorData,
        fate::{FateInstance, internally_start_fate},
        job_gauge::JobGauge,
        network::{DestinationNetwork, NetworkState},
        paralysis::Paralysis,
        periodic::PeriodicEffect,
        zone::Zone,
    },
    zone_connection::{BaseParameters, TeleportQuery},
};
use kawari::{
    common::{
        CharacterMode, DistanceRange, ENTRANCE_CIRCLE_IDS, EventState, FateState, HandlerId,
        HandlerType, MAXIMUM_FATES, MOB_WANDER_TIME, ObjectId, ObjectTypeId, Position,
        timestamp_secs,
    },
    config::{Config, get_config},
    ipc::zone::{
        ActionRequest, ActorControlCategory, Conditions, DutyFinderSetting, ObjectKind,
        ServerZoneIpcSegment, SpawnNpc, SpawnObject, SpawnPlayer, SpawnTreasure,
    },
};
use parking_lot::Mutex;

#[derive(Default, Debug)]
pub enum NavmeshGenerationStep {
    /// No generation is currently happening.
    #[default]
    None,
    /// We need to generate a navmesh at this path.
    Needed(String),
    /// The process to write the navmesh has started, and we need to wait until the file exists.
    Started(String),
}

/// CancelCast's second value for a quest's progress bar (an action's is 1, an item's 2, a mount's 13.)
const EVENT_ACTION_CANCEL_KIND: u32 = 4;

/// Why a quest's progress bar was cut short. Retail (Bob's Chronofoil captures 2026-10-14 to 2026-11-01): 37 broken
/// off by moving, CancelCast `537, 4, bar, 0`; three by a hit, as its damage landed (its EffectResult), and one by
/// paralysis, `540, 4, bar, 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventActionInterruption {
    Moved,
    Hit,
    Paralysed,
}

impl EventActionInterruption {
    /// CancelCast's first and last values.
    fn cancel_cast(self) -> (u32, u32) {
        match self {
            Self::Moved => (537, 0),
            Self::Hit | Self::Paralysed => (540, 1),
        }
    }
}

#[derive(Debug, Clone)]
pub enum QueuedTaskData {
    CastAction {
        request: ActionRequest,
        /// Currently means if it has a cast bar.
        interruptible: bool,
    },
    LoseStatusEffect {
        effect_id: u16,
        effect_param: u16,
        effect_source_actor_id: ObjectId,
    },
    /// Tell everyone nearby an actor was defeated, and by whom (ActorControl Defeated, 2 s after it died, as retail.)
    SendDefeated {
        actor_id: ObjectId,
        killer: ObjectId,
    },
    /// Fade out a dead actor.
    DeadFadeOut {
        actor_id: ObjectId,
        respawn_layout_id: Option<u32>,
    },
    /// Despawn a dead actor.
    DeadDespawn {
        actor_id: ObjectId,
        respawn_layout_id: Option<u32>,
    },
    /// Complete a director's EventAction (row `id`) on `target`
    CastEventAction { target: ObjectId, id: u32 },
    /// Complete a quest's EventAction (its progress bar, EventAction row `id`), which moving, an enemy's attack or
    /// paralysis can interrupt (`Instance::interrupt_event_action`.)
    QuestEventAction { id: u32 },
    /// A key item thrown or placed on the ground (`action::start_event_item_on_ground`): its cast bar ends (`landed_on`
    /// None), then, if it landed on an event object, that object's quest event starts.
    EventItemOnGround {
        request: ActionRequest,
        position: Position,
        landed_on: Option<ObjectTypeId>,
    },
    /// Make a fish bite.
    FishBite,
    /// A player an action walks towards `towards` (`EffectsBuilder::walk_to_caster`): `from_actor_id`, the player, is
    /// sent `ForcedMoveTowards` and shown in `pose` (ActorControl 82) to everyone near.
    WalkToCaster {
        towards: ObjectId,
        speed: u8,
        pose: u32,
    },
    /// The end of `WalkToCaster`: `ForcedMoveEnd`, and the pose undone (82 with 0) if there was one.
    WalkToCasterEnd { pose: u32 },
    /// Seal a boss wall.
    SealBossWall { id: u32, place_name: u32 },
    /// Generically send a packet segment, only used for `do_change_zone`. Don't abuse this as a generic task, you almost certainly want to create a new variant.
    PacketSegment { segment: ServerZoneIpcSegment },
    /// A mount appearing under `from_actor_id` (its Mount packet), sent to everyone near, itself too (`action.rs`.)
    MountArrives { segment: ServerZoneIpcSegment },
    /// An action's result as its hit lands (EffectResultBasic), sent to everyone near `from_actor_id`, the target.
    EffectResultLands { segment: ServerZoneIpcSegment },
    /// Used by directors since its tough to fit this into the director logic.
    WarpToPopRange { id: u32 },
    /// Reset a player's action combo status.
    ResetCombo,
    /// An NPC leaves its battle stance, if it still has nothing to fight.
    SheatheWeapon,
    /// Respawn a new mob.
    RespawnMob { layout_id: u32 },
    /// Ends a FATE.
    EndFate { fate_id: u32 },
    /// Terminates a director.
    TerminateDirector { handler_id: HandlerId },
    /// An NPC's leap lands it beside `target` (`npc_behavior::NPC_LEAPS`.)
    LeapTo { target: ObjectId },
}

#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub point: Instant,
    pub from_id: ClientId,
    pub from_actor_id: ObjectId,
    pub data: QueuedTaskData,
}

impl PartialEq for QueuedTask {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point
            && self.from_id == other.from_id
            && self.from_actor_id == other.from_actor_id
    }
}

/// Weather changes every 8 Eorzean hours: 1400 real seconds, on the boundary (retail: 161 of 161 within 3 s.)
pub const WEATHER_WINDOW_SECONDS: u32 = 1400;

#[derive(Debug, Default)]
pub struct Instance {
    /// List of spawned actors.
    pub actors: HashMap<ObjectId, NetworkedActor>,
    /// The timed gathering points that are up, by layout ID: their actor and the window they're up for.
    pub timed_gathering_spawned: HashMap<u32, (ObjectId, u32)>,
    /// The navmesh data.
    pub navmesh: Navmesh,
    /// The zone's collision, which NPCs walk on as players do (`collision`; written with the navmesh.)
    pub collision: Option<Arc<crate::collision::Collision>>,
    /// The event objects (layout IDs) hidden or removed here, whose gimmicks (their shared groups' collision) are
    /// gone with them: see `set_gimmick_gone`.
    pub gimmicks_gone: HashSet<u32>,
    /// The event objects (layout IDs) the director has shown here, whose gimmicks nobody gets through (a boss room
    /// sealed): see `seal_gimmick`.
    pub gimmicks_sealed: HashSet<u32>,
    /// The zone data.
    pub zone: Zone,
    /// The zone's current weather.
    pub weather_id: u16,
    /// The 1400 s weather window `weather_id` was worked out for (Unix time / 1400.)
    pub weather_window: u32,
    /// The HaterList and EnmityList last sent to each player, so they're only sent when they change.
    pub last_enmity_lists:
        HashMap<ObjectId, (kawari::ipc::zone::HaterList, kawari::ipc::zone::EnmityList)>,
    /// The content finder condition associated with this instance.
    pub content_finder_condition_id: u16,
    /// The content ID for the content finder condition, if applicable.
    pub content_id: u16,
    /// If Some, then this is the path of the navmesh we need to generate.
    pub generate_navmesh: NavmeshGenerationStep,
    /// List of tasks that has to be executed an arbitrary point in the future.
    pub queued_task: Vec<QueuedTask>,
    /// Current directors of this instance.
    pub directors: Vec<DirectorData>,
    /// Whether enemy AI is currently disabled.
    pub enemy_ai_disabled: bool,
    /// Idle NPCs found unable to wander (off the navmesh), so each is only logged once.
    pub stuck_wanderers: std::collections::HashSet<ObjectId>,
    /// The party bots here, by actor ID (`server/party_bot.rs`.)
    pub party_bots: HashMap<ObjectId, crate::server::party_bot::PartyBot>,
    /// Players who came in through the Duty Finder alone, whose party is filled with bots once they've loaded in.
    pub pending_duty_bots: std::collections::HashSet<ObjectId>,
    /// Players who registered through Duty Support, with the members they chose (DawnQuestMember, in party slot
    /// order): their party NPCs come once they've loaded in (`server/party_npc.rs`.)
    pub pending_duty_support: HashMap<ObjectId, Vec<u8>>,
    /// Story lines the director queued for the party NPCs (`director:party_npc_yell`.)
    pub party_npc_yells: Vec<crate::server::party_npc::QueuedYell>,
    /// Where the party goes as an enemy casts what (`director:party_spots`), and where the director has sent it for a
    /// while (`director:party_positions`): retail's Duty Support members' spots (`dodge::authored_spot`.)
    pub party_spots: Vec<crate::server::dodge::AuthoredSpots>,
    pub party_cue: Option<crate::server::dodge::PartyCue>,
    /// How many have been tossed into the air here (`action::toss_target`): each toss carries its number, as retail's.
    pub tosses: u32,
    /// The world's bots shown here (`world_bots.rs`): the NPC tick leaves them alone, and enemies never go after them.
    pub world_bots: HashSet<ObjectId>,
    /// A duty that hasn't begun: its players are still in its opening (inside the entrance circle, which keeps them
    /// in until DutyCommence.) Until every player is ready (`ready_to_commence`), its enemies notice no one and act
    /// not, and the party's bots stand where they are and do nothing (Bob's bundle 20261111-112150: Garuda and the
    /// Duty Finder's bots fought while he watched the Howling Eye's opening.)
    pub awaiting_commence: bool,
    pub ready_to_commence: HashSet<ObjectId>,
    /// NPCs a director is sending along a line, hurting as they go (`director:sweep_bnpc`, `server/sweep.rs`.)
    pub sweeps: Vec<crate::server::sweep::Sweep>,
    /// Aggressive BNpcs (by layout ID) a director has told not to notice anyone who comes close: they fight only when
    /// hit or sent after someone (`director:set_notice`; Victory in Peril's boss through its cutscene.)
    pub unaware_npcs: std::collections::HashSet<u32>,
    /// Open-world NPCs and whom they couldn't get to, until when they leave them alone (`npc_behavior`.)
    pub unreachable: HashMap<(ObjectId, ObjectId), std::time::Instant>,
    /// NPCs a wall has stopped, and for how many behaviour ticks in a row (`npc_behavior`, `NPC_WALL_PATIENCE_TICKS`.)
    pub walled_ticks: HashMap<ObjectId, u32>,
    /// NPCs a director spawned hidden that come into view a little later (`director:appear_bnpc`.)
    pub pending_appearances: Vec<crate::server::director::PendingAppearance>,
    /// The GATE the Gold Saucer is showing (its screens, its entrance's NPC), see `server/gate.rs`.
    pub gate_shown: Option<crate::gate::GateShown>,
    /// NPCs a director holds (`director:hold_bnpc`): `npc_behavior` leaves them alone, so they do only what the
    /// director has them do (warps, jumps, actions), as retail's invisible boss helpers.
    pub held_npcs: std::collections::HashSet<ObjectId>,
    /// BNpcs a director keeps in front of another wherever it goes (`director:attach_bnpc`): each one's parent and how
    /// far before it (`director::keep_attached`.)
    pub attached_npcs: HashMap<ObjectId, (ObjectId, f32)>,
    /// BNpcs a director has made untargetable (`director:set_targetable`) and not targetable again: the party's bots
    /// leave them be, as retail's Duty Support members let Titan go as he leapt and while his Heart stood (Bob's
    /// Chronofoil capture 650dbfd8.)
    pub untargetable_npcs: HashSet<ObjectId>,
    /// The HP, MP and max MP last sent for each actor (`update_actor_hp_mp` sends only changes.)
    pub last_hp_mp: HashMap<ObjectId, (u32, u16, u16)>,
    /// Director tasks queued for later (`director:delay`), and when they're due.
    /// Each bot party's leader: where they stood last tick and the way their formation faces (`party_bot`: it turns
    /// with them only as they move.)
    pub formation_facings: HashMap<ObjectId, (glam::Vec3A, f32)>,
    /// Whether this instance is a trial (`GameData::content_is_trial`), once asked (`party_bot`'s boss fights.)
    pub trial: Option<bool>,
    /// NPCs a director keeps busy until when (`director:busy_bnpc`): they don't move or act on their own, though their
    /// timelines go on.
    pub busy_npcs: HashMap<ObjectId, std::time::Instant>,
    /// NPCs a director keeps where they stand until when (`director:root_bnpc`): they don't move, but act as ever (their
    /// timeline, their attacks.)
    pub rooted_npcs: HashMap<ObjectId, std::time::Instant>,
    pub later_director_tasks: Vec<(std::time::Instant, crate::server::director::LuaDirectorTask)>,
    /// The fight here is over (a quest battle won, `npc_behavior::end_fight`): NPCs don't take anyone on again, whatever
    /// hits or heals them.
    pub fight_over: bool,
    /// Opened coffers, by actor ID (`server/loot.rs`.)
    pub coffers: HashMap<ObjectId, crate::server::loot::Coffer>,
    /// What each coffer (by base ID) held on retail, one list per recorded opening (`director:treasure_loot`.)
    pub loot_pools: HashMap<u32, Vec<Vec<u32>>>,
    /// What each coffer may hold besides (`director:treasure_pool`): the wiki's list, the starting point recordings
    /// add to.
    pub loot_wiki_pools: HashMap<u32, Vec<u32>>,
    /// The next LootItem serial.
    pub loot_serial: u32,
    /// List of running/preparing FATEs.
    pub fates: Vec<FateInstance>,
    /// List of FATEs that are able to spawn.
    pub candiate_fates: Vec<u32>,
    /// The duty finder settings applied to this instance, if applicable.
    pub content_settings: Option<DutyFinderSetting>,
    /// The maximum synced level for this content, if applicable.
    pub synced_level: Option<u8>,
    /// The item level this content syncs to, if it has its own (otherwise the synced level's ParamGrow.ItemLevelSync.)
    pub synced_item_level: Option<u16>,
    /// Minimum IL (`DutyFinderSetting::MINIMUM_ITEM_LEVEL`): everyone's item level is brought down to the duty's
    /// minimum (ContentFinderCondition.ItemLevelRequired), with or without the level sync.
    pub minimum_item_level: Option<u16>,
    /// The party's loot rule (`duty_finder::LootRule`): the leader's when the party went in full, Normal when the Duty
    /// Finder filled places (`duty_finder::loot_rule`.)
    pub loot_rule: crate::server::duty_finder::LootRule,
    /// The players' jumps and falls, for fall damage (`fall.rs`.)
    pub falls: HashMap<ObjectId, crate::server::fall::FallState>,
    /// The floors here that give a status (`status_floor.rs`), once found in the collision.
    pub status_floors: Option<Vec<crate::server::status_floor::StatusFloor>>,
    /// The status (and its param) each player has from the floor they stand on.
    pub on_status_floor: HashMap<ObjectId, (u16, u16)>,
    /// The battle NPCs of the zone's own layout, spawned as the instance was made (not by its director): in a duty
    /// they drop gil when another player is there (`actor::kill_gil`.)
    pub layout_npcs: HashSet<ObjectId>,
    /// The players here who hadn't cleared this duty before (`ToServer::DutyNewcomer`): with one in the party everyone
    /// gets the duty's first-timer bonus when it's cleared.
    pub newcomers: HashSet<ObjectId>,
    /// The Echo and Epic Echo here, in percent (`duty_finder::refresh_echo`), and the multiplier last given the bots.
    pub echo: u16,
    pub epic_echo: u16,
    pub echo_applied: f32,
    /// When the party's current fight began (an enemy hating one of them), for The Echo's three minutes.
    pub fight_started: Option<std::time::Instant>,
    /// How long this content lasts for, if applicable.
    pub duration: Option<Duration>,
    /// Players who helped defeat each running FATE's enemies (by FATE), who get its rewards if it's completed.
    pub fate_participants: HashMap<u32, Vec<ObjectId>>,
    /// When the duty's clock started, and when its time left was last re-sent (retail re-sends it every 300 s.)
    pub duty_clock: Option<(Instant, Instant)>,
    /// Which of the duty's two time flags its players have been sent (bits; `director::DUTY_FLAG_TIMES`.)
    pub duty_flags: u8,
    /// A party wipe under way (`server/wipe.rs`.)
    pub party_wipe: Option<crate::server::wipe::PartyWipe>,
    /// Where players start in a quest battle, and which way they face.
    pub quest_battle_start: Option<(Position, f32)>,
    /// Where the players go when they leave a won quest battle, if not back where they came from (the script's
    /// `EXIT_AFTER_WIN`: territory, position, rotation.)
    pub quest_battle_win_exit: Option<(u16, Position, f32)>,
    /// Running over-time effects (DoTs, HoTs, etc.)
    pub periodic_effects: Vec<PeriodicEffect>,
    /// Barriers on actors, by the status holding each (`server/barrier.rs`.)
    pub barriers: crate::server::barrier::Barriers,
    /// Who is paralysed: their rolls, and NPCs that just seized up (`server/paralysis.rs`.)
    pub paralysis: Paralysis,
    /// Battle NPCs a quest brought out, which don't respawn once defeated.
    pub quest_spawned_npcs: HashSet<ObjectId>,
    /// When players' HP and MP next regenerate (every 3 s, like retail's tick.)
    pub next_regen_tick: Option<std::time::Instant>,
}

impl Instance {
    pub fn new(
        id: u16,
        game_data: &mut GameData,
        lua: &KawariLua,
        explorer_mode: bool,
        allow_special_fates: bool,
    ) -> Self {
        let mut instance = Instance {
            zone: Zone::load(game_data, id),
            weather_id: game_data.get_weather(id as u32).unwrap_or_default() as u16,
            weather_window: timestamp_secs() / WEATHER_WINDOW_SECONDS,
            ..Default::default()
        };

        let config = get_config();
        if config.filesystem.navimesh_path.is_empty() {
            tracing::warn!("Navimesh path is not set! Monsters will not function correctly!");
        } else if instance.zone.navimesh_path.is_empty() {
            tracing::warn!("No navimesh path for this zone, skipping generation!");
        } else {
            let mut nvm_path = PathBuf::from(config.filesystem.navimesh_path);
            nvm_path.push(instance.zone.navimesh_path.clone());

            if let Ok(nvm_bytes) = std::fs::read(&nvm_path) {
                if let Some(navmesh) = Navmesh::from_existing(&nvm_bytes) {
                    instance.navmesh = navmesh;

                    tracing::info!("Successfully loaded navimesh from {nvm_path:?}");
                } else {
                    tracing::warn!(
                        "Failed to read {nvm_path:?}, monsters will not function correctly!"
                    );
                }
                instance.collision = crate::collision::Collision::load_shared(&nvm_path);
                // A zone with floors that give a status needs its collision's materials (version 3): an older one
                // is made again.
                let needs_materials = crate::server::status_floor::has_floors(instance.zone.id)
                    && instance
                        .collision
                        .as_ref()
                        .is_some_and(|collision| !collision.has_materials());
                if needs_materials {
                    tracing::info!(
                        "{nvm_path:?}'s collision has no materials (for its status floors), so it's made again"
                    );
                }
                // A navmesh made before the collision was written with it: made again (both), keeping this one
                // until then. NPCs keep to the navmesh's floor meanwhile.
                // One made before the collision had the layouts' simple shapes (version 4) or left open doors open
                // (version 5), or beside a navmesh with 32-bit polygon references (version 6: that navmesh isn't read):
                // made again too.
                let older = instance
                    .collision
                    .as_ref()
                    .is_some_and(|collision| collision.is_older());
                if older {
                    tracing::info!(
                        "{nvm_path:?}'s collision is from an older kawari-navimesh (before open doors and simple shapes), so it's made again"
                    );
                }
                if (instance.collision.is_none() || needs_materials || older)
                    && config.world.generate_navmesh
                {
                    if instance.collision.is_none() {
                        tracing::info!(
                            "{nvm_path:?} has no collision yet, so it's made again with one"
                        );
                    }
                    instance.generate_navmesh =
                        NavmeshGenerationStep::Needed(nvm_path.to_str().unwrap().to_string());
                }
            } else if config.world.generate_navmesh {
                instance.generate_navmesh =
                    NavmeshGenerationStep::Needed(nvm_path.to_str().unwrap().to_string());
            } else {
                tracing::warn!(
                    "Failed to read {nvm_path:?}, monsters will not function correctly!"
                );
            }
        }

        // Load initial event objects into instance
        for (object, layer_name) in instance.zone.get_event_objects(game_data, explorer_mode) {
            let hidden = object.event_state.contains(EventState::OFF);
            let layout_id = object.layout_id;
            instance.insert_object(object.entity_id, object, layer_name);
            if hidden {
                instance.set_gimmick_gone(layout_id, true);
            }
        }

        // Load initial NPCs into instance
        if !explorer_mode {
            let config = get_config();
            for npc in instance.zone.get_npcs(game_data) {
                let actor_id = ObjectId(fastrand::u32(..));
                instance.insert_npc(actor_id, npc, &config, game_data);
                instance.layout_npcs.insert(actor_id);
            }
        }

        // Determine the starting set of FATEs
        let available_fates: Vec<u32> = instance
            .zone
            .map_ranges
            .iter()
            .filter_map(|x| x.fate)
            .collect();

        // Filter all special event FATEs and chained ones for now.
        instance.candiate_fates = available_fates
            .iter()
            .copied()
            .filter(|fate| {
                let (chain, special) = game_data.get_fate_chain_special(*fate).unwrap_or_default();
                if !allow_special_fates && special {
                    return false;
                }
                chain == 0
            })
            .collect();

        if !instance.candiate_fates.is_empty() {
            instance.directors.push(DirectorData {
                id: HandlerId::new(HandlerType::Fate, 0xFFFF),
                ..Default::default()
            });
        }
        for fate_id in fastrand::choose_multiple(instance.candiate_fates.clone(), MAXIMUM_FATES) {
            let should_start;
            if let Some(fate) = FateInstance::new(fate_id, lua.clone(), game_data) {
                should_start = fate.fate_state == FateState::Running;
                instance.fates.push(fate);
            } else {
                should_start = false;
            }
            // TODO: meh i kind of hate that we do this. ideally we could read the faterule from the outside and determine how to start it. i dunno
            if should_start {
                internally_start_fate(&mut instance, fate_id);
            }
        }

        instance
    }

    pub fn find_actor(&self, id: ObjectId) -> Option<&NetworkedActor> {
        self.actors.get(&id)
    }

    pub fn find_actor_mut(&mut self, id: ObjectId) -> Option<&mut NetworkedActor> {
        self.actors.get_mut(&id)
    }

    pub fn insert_npc(
        &mut self,
        id: ObjectId,
        spawn: SpawnNpc,
        config: &Config,
        game_data: &mut GameData,
    ) {
        // Explorer Mode: "no enemies or obstacles" (FFXIV Console Games Wiki, *Duty Finder*): the zone's enemies were
        // left out as the instance was made (`Instance::new`), and its director's (bosses, adds) aren't spawned either.
        if self
            .content_settings
            .is_some_and(|settings| settings.contains(DutyFinderSetting::EXPLORER_MODE))
            && spawn
                .character_data_flags
                .contains(kawari::ipc::zone::CharacterDataFlag::HOSTILE)
        {
            return;
        }

        // Load drop-ins
        let mut timeline = serde_json::from_str(
            &std::fs::read_to_string(config.filesystem.locate_timeline_file("Default.json"))
                .unwrap(),
        )
        .unwrap();

        let mut search_dirs: Vec<String> = config
            .filesystem
            .additional_resource_paths
            .iter()
            .cloned()
            .map(|mut x| {
                x.push_str("/timelines/");
                x
            })
            .collect();
        search_dirs.push("resources/timelines/".to_string());

        'outer: for search_dir in search_dirs {
            let Ok(search_dir) = std::fs::read_dir(search_dir) else {
                continue;
            };

            for entry in search_dir.flatten() {
                if !entry
                    .file_name()
                    .to_str()
                    .unwrap_or_default()
                    .ends_with(&format!("_{}.json", spawn.common.base_id))
                {
                    continue;
                }

                if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                    timeline = serde_json::from_str(&contents).unwrap();
                    break 'outer;
                }
            }
        }

        // Its hitbox, which its areas reach out from: retail's (recorded), else its model's as the client sizes it. With
        // none, the Copperbell Mines' bosses' areas were a giant's hitbox smaller than the client drew them, and bots
        // dodged to spots still inside what Bob saw (Bob, 2026-10-26.)
        let mut timeline: kawari::common::Timeline = timeline;
        if timeline.hitbox_radius <= 0.0
            && let Some(radius) = game_data.npc_hitbox_radius(spawn.common.base_id)
        {
            timeline.hitbox_radius = radius;
        }

        // TODO: does classjob/tribe even apply to NPCs?
        let modifiers = game_data
            .get_class_job_modifiers(spawn.common.class_job as u32)
            .expect("Failed to read param grow");

        let tribe_row = game_data
            .get_tribe(spawn.common.look.tribe as u8)
            .expect("Failed to read tribe");

        let level = spawn.common.level;

        let param_grow = game_data
            .get_param_grow(level as u32)
            .expect("Failed to read param grow");

        let primary_stat = game_data
            .get_job_primary_stat(spawn.common.class_job as u16)
            .unwrap_or(1);

        let mut parameters = BaseParameters::default();
        parameters.perform_calculations(primary_stat, &tribe_row, &param_grow, &modifiers);
        parameters.calculate_potencies(&param_grow, Some(&modifiers));

        let position = spawn.common.position.0;
        let rotation = spawn.common.rotation;
        self.actors.insert(
            id,
            NetworkedActor::Npc {
                state: NpcState::natural_state_of(&spawn),
                navmesh_path: VecDeque::default(),
                navmesh_path_lerp: 0.0,
                navmesh_target: None,
                last_position: None,
                spawn,
                timeline,
                timeline_position: 0,
                timeline_checked: 0,
                timeline_anchor: None,
                timeline_hp_done: 0,
                enmity: HashMap::new(),
                autoattack_timing: 0,
                currently_invulnerable: false,
                status_effects: StatusEffects::default(),
                last_wander_timestamp: Instant::now()
                    + Duration::from_secs(fastrand::u64(0..MOB_WANDER_TIME.as_secs())),
                parameters,
                home: position,
                home_rotation: rotation,
                returning_home: false,
                cast: None,
                rotation_state: Default::default(),
                shouts_used: Default::default(),
                placement: Default::default(),
                shown_as_player: None,
            },
        );
    }

    pub fn generate_actor_id() -> ObjectId {
        // TODO: ensure we don't collide with another actor
        ObjectId(fastrand::u32(..))
    }

    /// Finds all (alive) players and NPCs. Returns their ids, positions and battalions.
    pub fn find_possible_enemies(&self) -> Vec<(ObjectId, Position, u8)> {
        self.actors
            .iter()
            .filter(|(_, y)| {
                matches!(y, NetworkedActor::Player { .. })
                    || matches!(y, NetworkedActor::Npc { .. })
            })
            .filter(|(_, y)| {
                y.get_common_spawn().health_points > 0
                    && y.get_common_spawn().mode != CharacterMode::Dead
                    // Nothing goes after a retainer called at a bell.
                    && y.get_common_spawn().object_kind != ObjectKind::Retainer
            })
            // Nor after the world's bots (`world_bots.rs`.)
            .filter(|(id, _)| !self.world_bots.contains(id))
            .map(|(x, y)| {
                (
                    *x,
                    y.get_common_spawn().position,
                    y.get_common_spawn().battalion,
                )
            })
            .collect()
    }

    pub fn insert_empty_actor(&mut self, actor_id: ObjectId) {
        if self.actors.contains_key(&actor_id) {
            return;
        }

        self.actors.insert(
            actor_id,
            NetworkedActor::Player {
                spawn: SpawnPlayer::default(),
                status_effects: StatusEffects::default(),
                teleport_query: TeleportQuery::default(),
                distance_range: DistanceRange::Normal,
                conditions: Conditions::default(),
                executing_gimmick_jump: false,
                inside_instance_exit: false,
                parameters: BaseParameters::default(),
                dueling_opponent_id: ObjectId::default(),
                remove_cooldowns: false,
                last_combo_action: 0,
                combo_sequence: 0,
                autoattack_target: None,
                autoattack_timing: 0,
                job_gauge: JobGauge::default(),
            },
        );
    }

    /// Spawns the zone's timed gathering points whose window has opened and takes away those whose window has
    /// closed. While one is up its args2 say so (bytes: up, up, the position shown, 0): the client shows unspoiled and
    /// legendary points only with both flags and at that position, ephemeral ones only with them (7.57
    /// `GatheringEventHandler.Setup` and `0x140E8E8F0`; Bob's Chronofoil captures had 01 01 nn 00 for the points that
    /// were up.) Which position a window uses is picked from the point and the window, the same for everyone.
    pub fn update_timed_gathering_points(&mut self, network: &mut NetworkState, unix_seconds: u64) {
        if self.zone.timed_gathering_points.is_empty() {
            return;
        }
        let minute = crate::gathering::eorzea_minute_of_day(unix_seconds);
        for point in self.zone.timed_gathering_points.clone() {
            let layout = point.spawn.layout_id;
            let window = crate::gathering::window_start(&point.windows, minute);
            let spawned = self.timed_gathering_spawned.get(&layout).copied();
            if spawned.map(|(_, start)| start) == window {
                continue;
            }
            if let Some((actor_id, _)) = spawned {
                network.remove_actor(self, actor_id);
                self.timed_gathering_spawned.remove(&layout);
            }
            let Some(start) = window else {
                continue;
            };
            let args = point.spawn.args1.to_le_bytes();
            let positions = args[3].max(1) as u32;
            // The day's window number too, so the position changes from one day's window to the next.
            let day = unix_seconds / 4200;
            let shown =
                ((point.spawn.base_id as u64 + start as u64 + day) % positions as u64) as u8;
            let mut spawn = point.spawn;
            spawn.entity_id = ObjectId(fastrand::u32(..));
            spawn.args2 = u32::from_le_bytes([1, 1, shown, 0]);
            self.insert_object(spawn.entity_id, spawn, String::default());
            self.timed_gathering_spawned
                .insert(layout, (spawn.entity_id, start));
        }
    }

    /// An event object (by layout ID) hidden or removed here (`gone`), or shown again: the shared groups it controls
    /// (a barrier, a door, a boss room's wall) go or come back with it in this instance's collision, as the client
    /// takes them away for players. Bob's bundles of 2026-10-27: the Copperbell Mines' entrance barrier and the
    /// Tam-Tara Deepcroft's barriers held the bots up long after they were gone for him.
    pub fn set_gimmick_gone(&mut self, layout_id: u32, gone: bool) {
        if layout_id == 0 {
            return;
        }
        let known = if gone {
            !self.gimmicks_gone.insert(layout_id)
        } else {
            !self.gimmicks_gone.remove(&layout_id)
        };
        if known {
            return;
        }
        if gone {
            self.gimmicks_sealed.remove(&layout_id);
        }
        if let Some(collision) = &self.collision
            && collision.has_gimmick(layout_id)
        {
            let mut ours = (**collision).clone();
            ours.set_gimmick_gone(layout_id, gone);
            if gone {
                ours.set_gimmick_sealed(layout_id, false);
            }
            self.collision = Some(Arc::new(ours));
            tracing::info!(
                "The gimmick of event object {layout_id} (layout ID) is {} this instance's collision",
                if gone { "gone from" } else { "back in" }
            );
        }
    }

    /// An event object (by layout ID) the director shows here (a boss room's wall going up): its gimmick is back in the
    /// collision, and nobody gets through it (no `WALL_PATIENCE` way past for bots, as there's none for players.)
    pub fn seal_gimmick(&mut self, layout_id: u32) {
        if layout_id == 0 {
            return;
        }
        self.set_gimmick_gone(layout_id, false);
        if !self.gimmicks_sealed.insert(layout_id) {
            return;
        }
        if let Some(collision) = &self.collision
            && collision.has_gimmick(layout_id)
        {
            let mut ours = (**collision).clone();
            ours.set_gimmick_sealed(layout_id, true);
            self.collision = Some(Arc::new(ours));
        }
    }

    /// A collision just loaded: the gimmicks already gone here go from it too.
    pub fn collision_loaded(&mut self) {
        // Its floors' materials are found again.
        self.status_floors = None;
        let Some(collision) = &self.collision else {
            return;
        };
        let gone: Vec<u32> = self
            .gimmicks_gone
            .iter()
            .copied()
            .filter(|layout_id| collision.has_gimmick(*layout_id))
            .collect();
        if gone.is_empty() && self.gimmicks_sealed.is_empty() {
            return;
        }
        let mut ours = (**collision).clone();
        for layout_id in &gone {
            ours.set_gimmick_gone(*layout_id, true);
        }
        for layout_id in &self.gimmicks_sealed {
            ours.set_gimmick_sealed(*layout_id, true);
        }
        self.collision = Some(Arc::new(ours));
        tracing::info!(
            "The gimmicks of {} hidden event objects are gone from this instance's collision",
            gone.len()
        );
    }

    pub fn insert_object(&mut self, actor_id: ObjectId, object: SpawnObject, layer_name: String) {
        self.actors
            .insert(actor_id, NetworkedActor::Object { object, layer_name });
    }

    pub fn insert_treasure(&mut self, actor_id: ObjectId, treasure: SpawnTreasure) {
        self.actors.insert(
            actor_id,
            NetworkedActor::Treasure {
                treasure,
                opened_at: None,
            },
        );
    }

    /// Inserts a new task into the queue, with a set `duration` and given `data`.
    pub fn insert_task(
        &mut self,
        from_id: ClientId,
        from_actor_id: ObjectId,
        duration: Duration,
        data: QueuedTaskData,
    ) {
        self.queued_task.push(QueuedTask {
            point: Instant::now() + duration,
            from_id,
            from_actor_id,
            data,
        });
    }

    /// Finds all tasks relevant to a given actor.
    pub fn find_tasks(&self, for_actor_id: ObjectId) -> Vec<QueuedTask> {
        self.queued_task
            .iter()
            .filter(|x| x.from_actor_id == for_actor_id)
            .cloned()
            .collect()
    }

    pub fn cancel_task(&mut self, network: Arc<Mutex<NetworkState>>, task: &QueuedTask) {
        // Delete the selected task:
        self.queued_task.retain(|x| x != task);

        // Then actually do the work:
        if let QueuedTaskData::CastAction { request, .. } = &task.data {
            cancel_action(
                network.clone(),
                self,
                task.from_id,
                task.from_actor_id,
                request.action_id,
            )
        }
    }

    /// Interrupts the quest event action (the progress bar for searching something) this player is in the middle of,
    /// if any, as retail does: everyone around sees it cancelled (ActorControl CancelCast from the player: retail's
    /// first value says why, its second is 4 for an event action), then the client is told so the connection ends the
    /// event (`FromServer::QuestEventActionInterrupted`.)
    pub fn interrupt_event_action(
        &mut self,
        network: &mut NetworkState,
        actor_id: ObjectId,
        why: EventActionInterruption,
    ) {
        let mut interrupted = Vec::new();
        self.queued_task.retain(|task| {
            if task.from_actor_id == actor_id
                && let QueuedTaskData::QuestEventAction { id } = task.data
            {
                interrupted.push((task.from_id, id));
                false
            } else {
                true
            }
        });
        let (unk1, unk3) = why.cancel_cast();
        for (from_id, action_id) in interrupted {
            network.send_ac_in_range_inclusive_instance(
                self,
                actor_id,
                ActorControlCategory::CancelCast {
                    unk1,
                    unk2: EVENT_ACTION_CANCEL_KIND,
                    action_id,
                    unk3,
                },
            );
            network.send_to(
                from_id,
                FromServer::QuestEventActionInterrupted,
                DestinationNetwork::ZoneClients,
            );
        }
    }

    // NOTE: this currently does *not* call cancel_action, so be careful if you're porting from cancel_task!
    pub fn retain_tasks(&mut self, f: impl Fn(&QueuedTask) -> bool) {
        // Delete the selected tasks
        self.queued_task.retain(f);
    }

    /// Cancels all queued actions for this actor.
    /// Cancels what `actor_id` is casting (its queued CastAction tasks), and nothing else: its statuses still run out.
    /// Returns the actions that were being cast.
    pub fn cancel_actor_casts(&mut self, actor_id: ObjectId) -> Vec<u32> {
        let mut cancelled = Vec::new();
        self.queued_task.retain(|task| {
            if task.from_actor_id == actor_id
                && let QueuedTaskData::CastAction { request, .. } = &task.data
            {
                cancelled.push(request.action_id);
                false
            } else {
                true
            }
        });
        cancelled
    }

    pub fn cancel_actor_tasks(&mut self, actor_id: ObjectId) {
        // Delete the selected task:
        self.queued_task.retain(|x| x.from_actor_id != actor_id);
    }

    /// Returns the actor ID (if any) of the spawned EObj by it's instance ID in the layout.
    pub fn find_object(&self, layout_id: u32) -> Option<ObjectId> {
        for (id, actor) in &self.actors {
            if let NetworkedActor::Object { object, .. } = actor
                && object.layout_id == layout_id
            {
                return Some(*id);
            }
        }

        None
    }

    /// Returns the actor ID (if any) of the spawned BNpc by it's instance ID in the layout.
    pub fn find_npc(&self, layout_id: u32) -> Option<(ObjectId, Position)> {
        for (id, actor) in &self.actors {
            if let NetworkedActor::Npc { spawn, .. } = actor
                && spawn.common.layout_id == layout_id
            {
                return Some((*id, spawn.common.position));
            }
        }

        None
    }

    /// Returns the actor ID (if any) of the spawned EObj by it's EObj ID.
    pub fn find_object_by_eobj_id(&self, eobj_id: u32) -> Option<ObjectId> {
        for (id, actor) in &self.actors {
            if let NetworkedActor::Object { object, .. } = actor
                && object.base_id == eobj_id
            {
                return Some(*id);
            }
        }

        None
    }

    /// Returns the actor ID (if any) of a spawned EObj by its EObj ID or, if none has that, its layout ID: some
    /// duties' boss walls share their EObj ID with a line beside them (the Copperbell Mines' first two), so a script
    /// tells them apart by layout ID.
    pub fn find_object_by_eobj_or_layout_id(&self, id: u32) -> Option<ObjectId> {
        self.find_object_by_eobj_id(id).or_else(|| {
            self.actors
                .iter()
                .find_map(|(actor_id, actor)| match actor {
                    NetworkedActor::Object { object, .. } if object.layout_id == id => {
                        Some(*actor_id)
                    }
                    _ => None,
                })
        })
    }

    /// Returns the actor ID (if any) of the spawned EObj by it's EObj ID and layer name.
    pub fn find_object_by_eobj_id_and_layer_name(
        &self,
        eobj_id: u32,
        eq_layer_name: &str,
    ) -> Option<ObjectId> {
        for (id, actor) in &self.actors {
            if let NetworkedActor::Object { object, layer_name } = actor
                && object.base_id == eobj_id
                && layer_name == eq_layer_name
            {
                return Some(*id);
            }
        }

        None
    }

    /// Returns the entrance circle event object (if found).
    pub fn find_entrance_circle(&self) -> Option<ObjectId> {
        // Prefer EObjs in LVD_zone_01 as that's where the circle is usually placed, otherwise other EObjs conflict such as in E8N.
        for base_id in ENTRANCE_CIRCLE_IDS {
            if let Some(id) = self.find_object_by_eobj_id_and_layer_name(base_id, "LVD_zone_01") {
                return Some(id);
            }
        }

        // Fallback to not matching by layer name...
        for base_id in ENTRANCE_CIRCLE_IDS {
            if let Some(id) = self.find_object_by_eobj_id(base_id) {
                return Some(id);
            }
        }

        None
    }

    /// Returns the base ID of the spawned EObj by it's actor ID.
    pub fn find_base_id_by_actor_id(&self, actor_id: ObjectId) -> Option<u32> {
        for (id, actor) in &self.actors {
            if *id == actor_id
                && let NetworkedActor::Object { object, .. } = actor
            {
                return Some(object.base_id);
            }
        }

        None
    }

    /// Returns the actor ID (if any) of the spawned EObj by it's Bind Layout ID.
    pub fn find_object_by_bind_layout_id(&self, bind_layout_id: u32) -> Option<ObjectId> {
        for (id, actor) in &self.actors {
            if let NetworkedActor::Object { object, .. } = actor
                && object.bind_layout_id == bind_layout_id
            {
                return Some(*id);
            }
        }

        None
    }

    /// Returns the FATE instance for this ID.
    pub fn find_fate_by_id_mut(&mut self, id: u32) -> Option<&mut FateInstance> {
        self.fates.iter_mut().find(|x| x.fate_id == id)
    }

    /// Returns the main (content) handler ID.
    /// For example, an instance with PublicContent and a FATE director would always return the PublicContent.
    pub fn content_handler_id(&self) -> Option<HandlerId> {
        self.directors
            .iter()
            .filter(|x| x.id.handler_type().is_content_director())
            .map(|x| x.id)
            .collect::<Vec<_>>()
            .first()
            .copied()
    }
}

/// Removes the actor from this instance, and terminates the directors from the client.
pub fn remove_actor_from_instance(
    data: &mut WorldServer,
    network: &mut NetworkState,
    actor_id: ObjectId,
) {
    // Their chocobo companion leaves with them (server/companion.rs.)
    crate::server::companion::owner_left(data, network, actor_id);
    // The bots of their party go with them (`party_bot::pack_for_travel`.)
    crate::server::party_bot::pack_for_travel(data, network, actor_id);
    // From every instance they're in: a player left behind in one (see `begin_change_zone`) was found there by the
    // next zone's loading, and got that instance instead of the one they went to.
    while let Some(current_instance) = data.find_actor_instance_mut(actor_id) {
        network.remove_actor(current_instance, actor_id);
        current_instance.last_enmity_lists.remove(&actor_id);

        // Take down the zone's FATEs first, as retail does: 2352 for each FATE's start NPC, then every FATE's context
        // (RemoveFateContext, for all the zone's FATEs, running or not), then the FATE director below. Without the
        // contexts going first, the client crashed in its FATE update after teleporting away (2026-10-17).
        for fate in &current_instance.fates {
            if let Some((npc, _)) = fate.motivation_npc {
                network.send_to_by_actor_id(
                    actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::UnkFate2 {
                        unk1: npc.0,
                        unk2: 0,
                        unk3: 0,
                        unk4: 0,
                    }),
                    DestinationNetwork::ZoneClients,
                );
            }
        }
        let mut fate_ids: Vec<u32> = current_instance.candiate_fates.clone();
        for fate in &current_instance.fates {
            if !fate_ids.contains(&fate.fate_id) {
                fate_ids.push(fate.fate_id);
            }
        }
        for fate_id in fate_ids {
            network.send_to_by_actor_id(
                actor_id,
                FromServer::ActorControlSelf(ActorControlCategory::RemoveFateContext { fate_id }),
                DestinationNetwork::ZoneClients,
            );
        }

        // Terminate any directors
        for director in &current_instance.directors {
            network.send_to_by_actor_id(
                actor_id,
                FromServer::ActorControlSelf(ActorControlCategory::TerminateDirector {
                    handler_id: director.id,
                }),
                DestinationNetwork::ZoneClients,
            );
        }
    }
}
