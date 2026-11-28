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
        ActionRequest, ActorControlCategory, Conditions, DisplayFlag, DutyFinderSetting,
        ObjectKind, ServerZoneIpcSegment, SpawnNpc, SpawnObject, SpawnPlayer, SpawnTreasure,
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
    /// A battle NPC a key item took (`event_item_effects.json`'s `captures`) goes: `actor::capture_battle_npc`.
    /// (`quietly`: gone without being shown defeated, `event_item_effects::Effects::quiet`.)
    Captured { actor_id: ObjectId, quietly: bool },
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
    /// A battle NPC's action landed on players (not missed or dodged): the director hears of it as the hit lands
    /// (`onActionLanded`: the clay golems' Sand Breath and True Grit, whose second line comes only when they hit.)
    ActionLanded {
        layout_id: u32,
        action_id: u32,
        players: u32,
    },
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
    /// Tells everyone here a FATE's state (FateInit): a boss FATE's 5 as its boss falls and 0 after it's over
    /// (`fate::BOSS_ENDING_AFTER`.)
    FateState {
        fate_id: u32,
        state: kawari::common::FateState,
    },
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
    /// The players a director's cutscene (`play_cutscene`) went to who haven't finished it yet: its script hears of
    /// its end (`onSceneFinished`) once the last has (Bob's mark of 2026-11-26 in the Praetorium: "bots did not wait for
    /// all players to leave the cutscene before attacking": the last boss came at the first player's end.)
    pub cutscene_watchers: HashSet<ObjectId>,
    /// The zone data.
    pub zone: Zone,
    /// The zone's current weather.
    pub weather_id: u16,
    /// Whether a director has forced `weather_id` (`set_weather`'s `forced`), to be given back when its battle is over
    /// (`director::restore_weather`.)
    pub weather_forced: bool,
    /// The 1400 s weather window `weather_id` was worked out for (Unix time / 1400.)
    pub weather_window: u32,
    /// Each thaumaturge NPC's Astral Fire or Umbral Ice (`thunderhead.rs`.)
    pub npc_elements: HashMap<ObjectId, crate::server::thunderhead::Element>,
    /// How many enemies have had each player on their list since the player came (`enemy_list_joins`.)
    pub enemies_joined: HashMap<ObjectId, u32>,
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
    /// Players a director moved (`warp_player_within`: PrepareZoning, then their client's ZoneIn), until they're in, at
    /// most `ARRIVAL_PATIENCE`: meanwhile everyone waits, as before the duty begins (`everyone_waits`).
    pub arriving: HashMap<ObjectId, std::time::Instant>,
    /// NPCs a director is sending along a line, hurting as they go (`director:sweep_bnpc`, `server/sweep.rs`.)
    pub sweeps: Vec<crate::server::sweep::Sweep>,
    /// Aggressive BNpcs (by layout ID) a director has told not to notice anyone who comes close: they fight only when
    /// hit or sent after someone (`director:set_notice`; Victory in Peril's boss through its cutscene.)
    pub unaware_npcs: std::collections::HashSet<u32>,
    /// Boss rooms whose NPCs keep to them (`director:keep_in_room`, `Room`.)
    pub rooms: Vec<Room>,
    /// Open-world NPCs and whom they couldn't get to, until when they leave them alone (`npc_behavior`.)
    pub unreachable: HashMap<(ObjectId, ObjectId), std::time::Instant>,
    /// NPCs a wall has stopped, and for how many behaviour ticks in a row (`npc_behavior`, `NPC_WALL_PATIENCE_TICKS`.)
    pub walled_ticks: HashMap<ObjectId, u32>,
    /// NPCs a director spawned hidden that come into view a little later (`director:appear_bnpc`.)
    pub pending_appearances: Vec<crate::server::director::PendingAppearance>,
    /// NPCs an action's knockback is throwing, until they've landed (`action::knock_back_npc`.)
    pub knockbacks: Vec<crate::server::action::Knockback>,
    /// Keybound Brawler's game, in its duty (`server/keybound.rs`.)
    pub keybound: Option<crate::server::keybound::Game>,
    /// The GATE the Gold Saucer is showing (its screens, its entrance's NPC), see `server/gate.rs`.
    pub gate_shown: Option<crate::gate::GateShown>,
    /// NPCs a director holds (`director:hold_bnpc`): `npc_behavior` leaves them alone, so they do only what the
    /// director has them do (warps, jumps, actions), as retail's invisible boss helpers.
    pub held_npcs: std::collections::HashSet<ObjectId>,
    /// NPCs whose director has stopped their auto-attacks (`director:hold_attacks`): they cast and move as ever.
    pub attacks_held: std::collections::HashSet<ObjectId>,
    /// BNpcs a director keeps in front of another wherever it goes (`director:attach_bnpc`): each one's parent and how
    /// far before it (`director::keep_attached`.)
    pub attached_npcs: HashMap<ObjectId, (ObjectId, f32)>,
    /// BNpcs a director has made untargetable (`director:set_targetable`) and not targetable again: the party's bots
    /// leave them be, as retail's Duty Support members let Titan go as he leapt and while his Heart stood (Bob's
    /// Chronofoil capture 650dbfd8.)
    pub untargetable_npcs: HashSet<ObjectId>,
    /// The party's NPCs a status holds (`modifiers::is_incapacitated`: a gaol), shown untargetable while it does, as
    /// the players are (ActorControl 54: retail's three Duty Support members with the player, each time the Porta
    /// Decumana's first boss gaoled the party, Bob's Chronofoil captures 9ba7d981, 6320ae56 and 50e72631.)
    pub incapacitated_party: HashSet<ObjectId>,
    /// BNpcs a director has the party's damage dealers go for first (`director:focus_bnpc`), as retail's Duty Support
    /// went for the Praetorium's Death Claw: its damage dealer hit each 2.3 and 4.8 s after it came, and both fell in
    /// 8 to 12 s (Bob's Chronofoil capture 35e30205; Bob, 2026-11-26: "dps should turn focus to it".)
    pub party_focus: HashSet<ObjectId>,
    /// How each enemy has taken players' statuses so far: Stun for less each time, then not at all
    /// (`resistance::Resistance`.)
    pub status_resistance: crate::server::resistance::Resistance,
    /// The HP, MP and max MP last sent for each actor (`update_actor_hp_mp` sends only changes.)
    pub last_hp_mp: HashMap<ObjectId, (u32, u16, u16)>,
    /// Director tasks queued for later (`director:delay`), and when they're due.
    /// Each bot party's leader: where they stood last tick and the way their formation faces (`party_bot`: it turns
    /// with them only as they move.)
    pub formation_facings: HashMap<ObjectId, (glam::Vec3A, f32)>,
    /// Where each bot party's leader has walked lately (`party_bot::FOOTSTEP_EVERY` apart, the last
    /// `party_bot::FOOTSTEPS`): ground the client let a player cross, whatever the zone's collision has there. A bot held
    /// up on its way follows them along it (Bob's marks of 2026-11-20 in Castrum Meridianum: "player NPCs couldn't path
    /// through wall barrier", the way into the first boss's room that map effect 1 opens, which the collision and
    /// navmesh were built with.)
    pub footsteps: HashMap<ObjectId, std::collections::VecDeque<glam::Vec3A>>,
    /// Whether this instance is a trial (`GameData::content_is_trial`), once asked (`party_bot`'s boss fights.)
    pub trial: Option<bool>,
    /// NPCs a director keeps busy until when (`director:busy_bnpc`): they don't move or act on their own, though their
    /// timelines go on.
    pub busy_npcs: HashMap<ObjectId, std::time::Instant>,
    /// NPCs whose timelines a director holds until when (`director:hold_timeline`): what comes due waits until then,
    /// while they move and auto-attack as ever.
    pub timeline_held: HashMap<ObjectId, std::time::Instant>,
    /// Attacks only shown (`director:show_attacks`): each (attacker, target) here, every hit of the attacker's on the
    /// target lands as a miss and does nothing, as retail's Cid's 29159 and 29160 on Castrum Meridianum's
    /// 9091115 (every one of 266 in four runs a Miss, its health untouched.)
    pub shown_attacks: HashSet<(ObjectId, ObjectId)>,
    /// NPCs a director keeps where they stand until when (`director:root_bnpc`): they don't move, but act as ever (their
    /// timeline, their attacks.)
    pub rooted_npcs: HashMap<ObjectId, std::time::Instant>,
    /// BNpcs that hold a guard up in front of them (`director:guard_front`), with how far to each side of their
    /// facing it reaches (radians): a hit from inside it is blocked for nothing (`action::guards_against`.)
    pub front_guards: HashMap<ObjectId, f32>,
    /// Where an NPC's action at no one is aimed, by NPC and action (`director:aim_action`): the spot its landing
    /// names, where the client plays it to (a leap's landing: Cape Westwind's boss's 28872.)
    pub action_aims: HashMap<(ObjectId, u32), glam::Vec3A>,
    /// Where a player last aimed an action at the ground (ActionRequestGroundTargeted), by player and action: its
    /// area's middle and the spot its AoeEffect names, as retail's were (the Praetorium's armour's 1128, Bob's
    /// Chronofoil capture 6320ae56.)
    pub ground_aims: HashMap<(ObjectId, u32), glam::Vec3A>,
    /// The players whose client has asked for the director's variables (ClientTrigger ReadyForDirectorVars, after its
    /// EnterTerritoryEvent): retail sends DirectorVars to none before (`Instance::vars_ready`.)
    pub director_vars_ready: HashSet<ObjectId>,
    /// Each launcher's turn (by layout ID) while its party is near (`launcher`.)
    pub launchers: HashMap<u32, crate::server::launcher::LauncherState>,
    /// Launchers (by layout ID) a director has stopped (`director:launcher`): there, but they don't cast.
    pub launchers_off: std::collections::HashSet<u32>,
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
    /// Every player being down isn't a wipe just now (`director:hold_wipe`): the fight itself fells them.
    pub wipe_held: bool,
    /// Where players start in a quest battle, and which way they face.
    pub quest_battle_start: Option<(Position, f32)>,
    /// NPCs that damage leaves at this much health at the least (`director:set_health_floor`): Cape Westwind's boss
    /// stopped at 1 HP twice, to go on with the next part of the fight (Bob's Chronofoil capture 667a68c5.)
    pub health_floors: HashMap<ObjectId, u32>,
    /// The quest battle begins with the Echo (Very Easy after a loss: `director::give_quest_battle_echo`.)
    pub quest_battle_echo: bool,
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

        instance.zone.note_layer_eobjs();
        let config = get_config();
        if config.filesystem.navimesh_path.is_empty() {
            tracing::warn!("Navimesh path is not set! Monsters will not function correctly!");
        } else if instance.zone.navimesh_path.is_empty() {
            tracing::warn!("No navimesh path for this zone, skipping generation!");
        } else {
            let mut nvm_path = PathBuf::from(&config.filesystem.navimesh_path);
            nvm_path.push(instance.zone.navimesh_path.clone());
            // Its own file isn't made yet (`Zone::territory_navmesh_path`, 2026-11-14): the level's old one, built
            // from every territory's layers, serves until it is, so no one stands about meanwhile.
            let mut old_path = PathBuf::from(&config.filesystem.navimesh_path);
            old_path.push(instance.zone.shared_navimesh_path.clone());
            let stand_in = !instance.zone.shared_navimesh_path.is_empty()
                && !nvm_path.exists()
                && old_path.exists();
            let wanted_path = nvm_path.clone();
            if stand_in {
                tracing::info!(
                    "{wanted_path:?} (territory {}'s own navmesh) isn't made yet: using {old_path:?} until it is",
                    instance.zone.id
                );
                nvm_path = old_path;
            }

            if let Ok(nvm_bytes) = std::fs::read(&nvm_path) {
                if let Some(navmesh) = Navmesh::from_existing(&nvm_bytes) {
                    instance.navmesh = navmesh;

                    tracing::info!("Successfully loaded navimesh from {nvm_path:?}");
                } else {
                    tracing::warn!(
                        "Failed to read {nvm_path:?}, monsters will not function correctly!"
                    );
                }
                let barriers = crate::map_effect_barriers::indices(instance.zone.id);
                instance.collision = crate::collision::Collision::load_shared(&nvm_path, &barriers);
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
                // (version 5), or beside a navmesh with 32-bit polygon references (version 6: that navmesh isn't read),
                // or with a duty's map effect barriers built in (version 7, `map_effect_barriers`): made again too.
                // Or one built for other map effect barriers than the zone's (`barriers_differ`): from a build that
                // couldn't find them (version 7: Castrum Meridianum's, Bob's marks of 2026-11-22), or before they
                // were listed in `map_effect_barriers.json` (the Praetorium's, Bob's marks of 2026-11-24.)
                // Or one whose navmesh left out the floors of event objects never spawned here (version 10: the
                // Praetorium's last boss's, `Collision::lacks_unspawned_floors`.)
                let older = instance.collision.as_ref().is_some_and(|collision| {
                    collision.is_older()
                        || collision.barriers_differ(&barriers)
                        || collision.lacks_unspawned_floors(&instance.zone.layer_eobjs)
                });
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
            if stand_in && config.world.generate_navmesh {
                instance.generate_navmesh =
                    NavmeshGenerationStep::Needed(wanted_path.to_str().unwrap().to_string());
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
        instance.mark_unspawned_gimmicks();

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
                pending_action: None,
                placement: Default::default(),
                shown_as_player: None,
            },
        );
    }

    /// An actor ID as retail gives what the server spawns: 0x40000000 and up, one after another (Gaius 0x400024A1 and
    /// the armour 0x400024B1 in the Praetorium, Bob's Chronofoil capture 35e30205; every NPC and object in his
    /// captures). A random u32 fell in the client's own range (0xE0000000 and up) one time in eight: two of the
    /// Praetorium's event objects in Bob's run of 2026-11-25 (0xF19825CB, 0xFE388158).
    pub fn generate_actor_id() -> ObjectId {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(1);
        let next = NEXT.fetch_add(1, Ordering::Relaxed) & 0x0FFF_FFFF;
        ObjectId(0x4000_0000 | next.max(1))
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

    /// Whether the enemies and the party's bots wait: the duty hasn't begun (`awaiting_commence`), or a player a
    /// director moved is still in their loading screen (`arriving`).
    pub fn everyone_waits(&self) -> bool {
        let now = std::time::Instant::now();
        self.awaiting_commence || self.arriving.values().any(|until| *until > now)
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

    /// The collision's gimmicks of event objects that aren't in this zone's layers that are on (`Zone::layer_eobjs`)
    /// are as the client has them, never spawned: their walls are no obstacle, their floors stay
    /// (`Collision::set_gimmick_unspawned`). The Praetorium's collision has gimmicks from before its remake (layers in
    /// no layer set of its own): 4305499's barrier (2000824) at (-184.6, -2.1) held the bots up on the way west until
    /// `WALL_PATIENCE` ran out (Bob's mark of 2026-11-26: "bots got stuck on barrier untill the 2 second fallback kicked
    /// in"), where nobody else was stopped; the last boss's floor is 4334405's gimmick, which everyone stands on.
    pub fn mark_unspawned_gimmicks(&mut self) {
        let Some(collision) = &self.collision else {
            return;
        };
        let unspawned: Vec<u32> = collision
            .gimmicks()
            .iter()
            .copied()
            .filter(|layout_id| *layout_id != 0 && !self.zone.layer_eobjs.contains(layout_id))
            .collect();
        if unspawned.is_empty() {
            return;
        }
        let mut ours = (**collision).clone();
        for layout_id in &unspawned {
            ours.set_gimmick_unspawned(*layout_id, true);
        }
        self.collision = Some(Arc::new(ours));
        tracing::info!(
            "The gimmicks of event objects {unspawned:?} (layout IDs), never spawned here, have their floors and no walls in this instance's collision"
        );
    }

    /// A collision just loaded: the gimmicks already gone here go from it too.
    pub fn collision_loaded(&mut self) {
        self.mark_unspawned_gimmicks();
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

    /// Every NPC of a layout (a director may spawn one layout more than once: `director:spawn_tethered_bnpc`.)
    pub fn find_npcs(&self, layout_id: u32) -> Vec<ObjectId> {
        self.actors
            .iter()
            .filter(|(_, actor)| {
                matches!(actor, NetworkedActor::Npc { spawn, .. } if spawn.common.layout_id == layout_id)
            })
            .map(|(id, _)| *id)
            .collect()
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
    /// An EObj ID two objects here share names whichever comes first (the actors' order, which differs instance to
    /// instance): said, so the script names the one it means by layout ID (the Praetorium's 2000789, the first boss's
    /// door and the shortcut by the magitek terminal: the pull showed the shortcut, Bob's marks of 2026-11-26.)
    /// Every object of an EObj here, in the order they came (their actor IDs): the seven the last battle's helpers'
    /// 29329 leave, which go together.
    pub fn objects_of_eobj(&self, base_id: u32) -> Vec<ObjectId> {
        let mut ids: Vec<ObjectId> = self
            .actors
            .iter()
            .filter(|(_, actor)| matches!(actor, NetworkedActor::Object { object, .. } if object.base_id == base_id))
            .map(|(id, _)| *id)
            .collect();
        ids.sort_by_key(|id| id.0);
        ids
    }

    pub fn find_object_by_eobj_or_layout_id(&self, id: u32) -> Option<ObjectId> {
        let sharing = self
            .actors
            .values()
            .filter(|actor| matches!(actor, NetworkedActor::Object { object, .. } if object.base_id == id))
            .count();
        if sharing > 1 {
            tracing::warn!(
                "EObj ID {id} names {sharing} objects here: one of them is taken, any of them; a director should name it by layout ID"
            );
        }
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
    ///
    /// Fork: retail's circle for this territory first, by its layout ID (`entrance_circles.json`): Castrum
    /// Meridianum's way to its last hall shares its EObj ID (2007457) with the Porta Decumana's circle, and was hidden
    /// at DutyCommence in its circle's place (Bob's mark of 2026-11-19, "stuck in circular barrier").
    pub fn find_entrance_circle(&self) -> Option<ObjectId> {
        if let Some(layout_id) = crate::entrance_facing::entrance_circle(self.zone.id)
            && let Some(id) = self
                .actors
                .iter()
                .find_map(|(actor_id, actor)| match actor {
                    NetworkedActor::Object { object, .. } if object.layout_id == layout_id => {
                        Some(*actor_id)
                    }
                    _ => None,
                })
        {
            return Some(id);
        }
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

    /// The players a director's DirectorVars go to: those whose client has asked for them. Retail's first DirectorVars
    /// in the Praetorium came 1.5 s in, after the client's EnterTerritoryEvent and after the duty's event NPCs were
    /// spawned, in each of Bob's runs (50e72631, 6320ae56, 9ba7d981); Kawari sent one with the ZoneInit, before the
    /// client had loaded, from the script's `set_sequence(1)` at setup (Bob's run of 2026-11-25 on `23467df9a`.)
    pub fn vars_ready(&self) -> Vec<ObjectId> {
        self.actors
            .iter()
            .filter(|(id, actor)| {
                matches!(actor, NetworkedActor::Player { .. })
                    && self.director_vars_ready.contains(id)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// The layout's event NPCs of a duty are its director's, as retail spawned them: The Praetorium's armour
    /// (ENpcBase 1042680) and the others of its LVD_REMAKE_GIMMICK layers came with handler 0x80030056 and display
    /// flags 0x40008 (Bob's Chronofoil capture 50e72631, 2026-11-25).
    pub fn give_event_npcs_to(&mut self, handler_id: HandlerId) {
        for actor in self.actors.values_mut() {
            if let NetworkedActor::Npc { spawn, .. } = actor
                && spawn.common.object_kind == ObjectKind::EventNpc
            {
                spawn.common.handler_id = handler_id;
                spawn.common.display_flags = DisplayFlag::UNK1 | DisplayFlag::UNK2;
            }
        }
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
        current_instance.enemies_joined.remove(&actor_id);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn object(base_id: u32, layout_id: u32) -> SpawnObject {
        SpawnObject {
            base_id,
            layout_id,
            ..Default::default()
        }
    }

    /// Bob's mark of 2026-11-26 in the Praetorium: the gimmick of 4305499 (2000824), in a layer no layer set turns on,
    /// held the bots up on the way west until their patience ran out. A gimmick of an event object outside the zone's
    /// layers that are on has no walls (nobody was stopped by it), but its floors stay (the last boss's floor is the
    /// gimmick of 4334405, never spawned either); one inside stays whole until the director hides it.
    #[test]
    fn an_unspawned_gimmick_has_floors_but_no_walls() {
        use crate::collision::Collision;
        use glam::Vec3A;
        let quad = |v: &mut Vec<[f32; 3]>,
                    t: &mut Vec<[u32; 3]>,
                    a: [f32; 3],
                    b: [f32; 3],
                    c: [f32; 3],
                    d: [f32; 3]| {
            let i = v.len() as u32;
            v.extend([a, b, c, d]);
            t.extend([[i, i + 1, i + 2], [i, i + 2, i + 3]]);
        };
        let (mut v, mut t) = (Vec::new(), Vec::new());
        // The floor is 4334405's; walls across x = 3 (4305499's) and x = -3 (9247278's).
        quad(
            &mut v,
            &mut t,
            [-10.0, 0.0, -10.0],
            [10.0, 0.0, -10.0],
            [10.0, 0.0, 10.0],
            [-10.0, 0.0, 10.0],
        );
        quad(
            &mut v,
            &mut t,
            [3.0, 0.0, -10.0],
            [3.0, 0.0, 10.0],
            [3.0, 4.0, 10.0],
            [3.0, 4.0, -10.0],
        );
        quad(
            &mut v,
            &mut t,
            [-3.0, 0.0, -10.0],
            [-3.0, 0.0, 10.0],
            [-3.0, 4.0, 10.0],
            [-3.0, 4.0, -10.0],
        );
        let owners = vec![4334405, 4334405, 4305499, 4305499, 9247278, 9247278];
        let mut instance = Instance::default();
        instance.zone.layer_eobjs.insert(9247278);
        instance.collision = Some(Arc::new(Collision::build(
            v,
            t,
            owners,
            Vec::new(),
            0.75,
            0.4,
            55.0,
        )));
        instance.mark_unspawned_gimmicks();
        let collision = instance.collision.clone().unwrap();
        let east = collision.walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0);
        assert!(
            !east.blocked && (east.at.x - 8.0).abs() < 0.01 && east.at.y.abs() < 0.01,
            "{east:?}"
        );
        let west = collision.walk(Vec3A::ZERO, Vec3A::new(-8.0, 0.0, 0.0), 50.0);
        assert_eq!(west.gimmick, Some(9247278));
        assert_eq!(collision.settle(Vec3A::new(5.0, 0.3, 5.0)).y, 0.0);
        // A collision loaded again is marked too.
        instance.collision = Some(Arc::new(Collision::build(
            vec![
                [3.0, 0.0, -10.0],
                [3.0, 0.0, 10.0],
                [3.0, 4.0, 10.0],
                [3.0, 4.0, -10.0],
                [-10.0, 0.0, -10.0],
                [10.0, 0.0, -10.0],
                [10.0, 0.0, 10.0],
                [-10.0, 0.0, 10.0],
            ],
            vec![[0, 1, 2], [0, 2, 3], [4, 5, 6], [4, 6, 7]],
            vec![4305499, 4305499, 0, 0],
            Vec::new(),
            0.75,
            0.4,
            55.0,
        )));
        instance.collision_loaded();
        let east =
            instance
                .collision
                .clone()
                .unwrap()
                .walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0);
        assert!(!east.blocked, "{east:?}");
        // Put up by the director (a map effect's barrier, no event object's), it stands.
        instance.seal_gimmick(4305499);
        let sealed =
            instance
                .collision
                .clone()
                .unwrap()
                .walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0);
        assert!(sealed.blocked && sealed.sealed, "{sealed:?}");
    }

    /// Bob's mark of 2026-11-19 in Castrum Meridianum, "stuck in circular barrier": its way to the last hall (EObj
    /// 2007457, layout 9090412) shares its EObj ID with the Porta Decumana's circle, and was hidden at DutyCommence in
    /// place of the circle (2000562, layout 9088360; ActorControl 106 [7] 16.9 s in, the circle never). Retail's
    /// circle is recorded by layout ID (`entrance_circles.json`) and comes first.
    #[test]
    fn the_entrance_circle_is_retails_by_layout() {
        let mut instance = Instance::default();
        instance.zone.id = 1043;
        instance.insert_object(
            ObjectId(1),
            object(2007457, 9090412),
            "LVD_zone_01".to_string(),
        );
        instance.insert_object(
            ObjectId(2),
            object(2000562, 9088360),
            "LVD_zone_01".to_string(),
        );
        assert_eq!(instance.find_entrance_circle(), Some(ObjectId(2)));

        // A duty not recorded still goes by the EObj IDs.
        instance.zone.id = 9999;
        assert_eq!(instance.find_entrance_circle(), Some(ObjectId(1)));
    }
}

/// A boss room whose NPCs (by layout ID) keep to it (`director:keep_in_room`): they notice only whoever is inside,
/// are pulled together, and never step out after anyone. Castrum Meridianum's Black Eft and its four were pulled
/// within 2.1 s of each other in all five retail runs, as the player came 7 to 11 yalms inside the room's line (24
/// yalms out), and none went more than 10.7 yalms from the room's middle; Kawari's four noticed the party at the
/// room's door, 11 s before their boss, and walked out after it (Bob's mark of 2026-11-21: "enemies are not suppose
/// to leave the wall barrier").
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub middle: glam::Vec3A,
    pub radius: f32,
    pub members: HashSet<u32>,
}

/// How far inside its room's edge a member's chase stops (yalms): the edge itself is the wall.
pub const ROOM_MARGIN: f32 = 1.0;

impl Room {
    /// Whether `at` is inside the room (across the ground.)
    pub fn holds(&self, at: glam::Vec3A) -> bool {
        glam::Vec3A::new(at.x - self.middle.x, 0.0, at.z - self.middle.z).length() <= self.radius
    }

    /// `goal`, or where the room's edge (less `ROOM_MARGIN`) is on the way to it from the middle.
    pub fn keep_inside(&self, goal: glam::Vec3A) -> glam::Vec3A {
        let offset = glam::Vec3A::new(goal.x - self.middle.x, 0.0, goal.z - self.middle.z);
        let limit = (self.radius - ROOM_MARGIN).max(0.0);
        if offset.length() <= limit {
            return goal;
        }
        let inside = self.middle + offset.normalize_or_zero() * limit;
        glam::Vec3A::new(inside.x, goal.y, inside.z)
    }
}
