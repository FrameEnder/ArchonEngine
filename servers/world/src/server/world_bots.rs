//! Players about the world: bots shown as players standing round the big aetherytes and the marketboards, walking
//! between a zone's places, through its exits into the next zone, into an inn and out again, and teleporting between
//! aetherytes (docs/src/fork/world-bots.md). Off unless the server's owner turns them on (`world-bots.json` in the data
//! folder, ArchonEngine ControlPanel's Dashboard → World bots.)
//!
//! Every bot is kept here, wherever it is, whether anyone is in its zone or not: who it is, where it stands, what it's
//! doing and until when. In a zone a player is in, it's an actor of that zone's instance (an NPC shown as a player,
//! as party bots are: `party_bot::player_spawn`), walking on the navmesh as a party bot walks; elsewhere its time just
//! runs on (a walk takes as long as it would, a teleport its cast and a loading screen.) An instance's NPC tick leaves
//! its world bots alone (`Instance::world_bots`) and enemies never go after them.
//!
//! What they do comes from retail's players in Bob's Chronofoil captures (124 captures, 404 player-hours in the six
//! city zones; numbers only): how many stood in view, where (`resources/data/world_bot_spots.json`), how long they
//! stood, how often they changed their idle pose or used an emote, how they left (one in eight by Teleport: a 5 s
//! cast, gone 5.9 s after it began) and arrived (ZoneIn about a second after they appeared), their jobs, levels,
//! online statuses and titles.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use glam::{Affine3A, Vec3A};
use kawari::{
    common::{
        CharacterMode, JumpState, MoveAnimationState, MoveAnimationType, ObjectId, ObjectTypeId,
        ObjectTypeKind, Position, WarpType,
    },
    config::get_config,
    ipc::zone::{
        ActionType, ActorControlCategory, ActorSetPos, BattleNpcSubKind, CharacterDataFlag,
        CommonSpawn, DisplayFlag, ObjectKind, OnlineStatus, ServerZoneIpcData,
        ServerZoneIpcSegment, SpawnNpc,
    },
};
use parking_lot::Mutex;
use physis::{layer::LayerEntryData, savedata::chardat::CustomizeData};
use serde_json::Value;

use kawari::common::TerritoryIntendedUse;

use crate::{
    FromServer, GameData,
    gamedata::AetheryteRowPlace,
    inventory::EquippedStorage,
    server::{
        WorldServer,
        actor::{NetworkedActor, NpcState, set_character_mode},
        bot_friends::SavedBot,
        glamour,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
        npc_behavior::NPC_RUN_SPEED,
        party_bot::{self, ShownAsPlayer},
        zone::Zone,
    },
};

/// The settings' file name, in the server's data folder.
pub const SETTINGS_FILE: &str = "world-bots.json";

/// Where the settings are read from: `$KAWARI_DATA` (the container's `/kawari/data`), as `glamours.txt` is.
pub fn settings_path() -> PathBuf {
    match std::env::var_os("KAWARI_DATA") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join(SETTINGS_FILE),
        _ => PathBuf::from(SETTINGS_FILE),
    }
}

/// What the server's owner chose (`world-bots.json`):
///
/// ```json
/// {"format": 1, "enabled": true, "cities": {"129": 30, "130": 10, "132": 20}, "per_aetheryte": 1,
///  "per_marketboard": [5, 10], "travel": true}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    /// Territories whose big aetheryte gathers more bots than `per_aetheryte`: (TerritoryType, how many.)
    pub cities: Vec<(u16, u32)>,
    /// Bots round every other big aetheryte in a town or the open world.
    pub per_aetheryte: u32,
    /// Bots round each marketboard: between the two (both included), drawn for each board.
    pub per_marketboard: (u32, u32),
    /// Whether they walk between places, through exits, into inns and teleport (otherwise they stay put.)
    pub travel: bool,
}

/// Bob's numbers, 2026-11-11: about 30 round Limsa Lominsa's aetheryte (Lower Decks, 129), 10 round Ul'dah's (Steps of
/// Nald, 130), 20 round Gridania's (New Gridania, 132), 5-10 at each marketboard. Retail had a median of 57 other
/// players in view in the Lower Decks, 13 in the Steps of Nald and 14 in New Gridania (up to 62, 29 and 44.)
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            cities: vec![(129, 30), (130, 10), (132, 20)],
            per_aetheryte: 1,
            per_marketboard: (5, 10),
            travel: true,
        }
    }
}

impl Settings {
    /// The settings in the file's JSON; what's missing or wrong takes the default.
    pub fn from_json(value: &Value) -> Self {
        let mut settings = Settings::default();
        if let Some(enabled) = value.get("enabled").and_then(Value::as_bool) {
            settings.enabled = enabled;
        }
        if let Some(cities) = value.get("cities").and_then(Value::as_object) {
            settings.cities = cities
                .iter()
                .filter_map(|(territory, count)| {
                    Some((territory.parse().ok()?, count.as_u64()?.min(200) as u32))
                })
                .collect();
            settings.cities.sort();
        }
        if let Some(count) = value.get("per_aetheryte").and_then(Value::as_u64) {
            settings.per_aetheryte = count.min(20) as u32;
        }
        if let Some([low, high]) = value
            .get("per_marketboard")
            .and_then(Value::as_array)
            .map(|range| range.iter().filter_map(Value::as_u64).collect::<Vec<_>>())
            .as_deref()
        {
            let (low, high) = ((*low).min(50) as u32, (*high).min(50) as u32);
            settings.per_marketboard = (low.min(high), low.max(high));
        }
        if let Some(travel) = value.get("travel").and_then(Value::as_bool) {
            settings.travel = travel;
        }
        settings
    }
}

/// How long a player stood in one spot in each city zone, seconds (the 10th, 25th, 50th, 75th and 90th percentiles of
/// 10,723 stays of 8 s or more: Bob's Chronofoil captures.) Elsewhere, New Gridania's.
const STAY_SECONDS: [(u16, [f32; 5]); 6] = [
    (128, [10.5, 16.9, 39.3, 86.8, 304.8]),
    (129, [11.1, 21.2, 54.8, 163.7, 628.2]),
    (130, [10.6, 16.1, 34.2, 90.7, 307.8]),
    (131, [10.6, 17.1, 42.6, 117.0, 346.5]),
    (132, [11.4, 21.2, 58.8, 97.1, 167.9]),
    (133, [10.4, 15.3, 29.1, 75.4, 186.3]),
];
const STAY_ELSEWHERE: [f32; 5] = [11.4, 21.2, 58.8, 97.1, 167.9];

/// Per player-hour in the city zones (404 player-hours of Bob's captures): idle-pose changes (ActorControl 295, the
/// pose 0-6 about evenly) and emotes (ActorControlTarget 290.)
const POSE_CHANGES_PER_HOUR: f32 = 29.6;
const EMOTES_PER_HOUR: f32 = 2.9;
/// The emotes retail's players used in the cities: (emote, how often, the share in percent aimed at someone) (Bob's
/// Chronofoil captures, ActorControlTarget 290 with its target; `notes/world-bots/cityemotes.py`.) Only those that play
/// once are used (`GameData::one_shot_emotes`); an aimed one goes to another bot near it, or isn't used.
const EMOTES: [(u32, u32, u8); 30] = [
    (112, 151, 100),
    (146, 67, 100),
    (105, 43, 100),
    (211, 40, 92),
    (216, 37, 62),
    (205, 35, 14),
    (7, 28, 89),
    (52, 21, 33),
    (121, 18, 17),
    (50, 16, 0),
    (42, 16, 100),
    (113, 15, 100),
    (274, 15, 93),
    (291, 11, 100),
    (297, 11, 27),
    (5, 10, 80),
    (213, 10, 100),
    (103, 10, 10),
    (236, 9, 78),
    (6, 9, 89),
    (16, 9, 100),
    (238, 9, 11),
    (154, 8, 25),
    (203, 8, 75),
    (149, 7, 86),
    (23, 7, 100),
    (292, 7, 71),
    (126, 6, 50),
    (41, 6, 100),
    (252, 6, 100),
];
/// Sitting, from Bob's captures (`notes/world-bots/citysit.py`): in a chair, the emote Sit (50) then SetMode 11 with 2;
/// on the ground, the emote Groundsit (52) then SetMode 11 with 1 (EmoteMode rows.) 4% of retail's players were in a
/// chair as they came into view, and 3% in a looped emote; how often a bot goes to sit is a guess near those.
const SIT_EMOTE: u32 = 50;
const GROUND_SIT_EMOTE: u32 = 52;
/// Getting up: from a chair (51), from the ground (53).
const STAND_EMOTE: u32 = 51;
const GROUND_STAND_EMOTE: u32 = 53;
/// Retail's ActorSetPos into and out of a seat: warp type 19, with 1 sitting down and 2 getting up; and where they're
/// put getting up: 0.42 yalms ahead of the seat, half a yalm down (all four seen.)
const SEAT_WARP_SIT: u8 = 1;
const SEAT_WARP_STAND: u8 = 2;
const SEAT_STAND_AHEAD: f32 = 0.42;
const SEAT_STAND_DOWN: f32 = 0.5;
/// How long a bot just arrived (by Teleport or through an exit) stands where it arrived before walking off to a spot
/// of its own (seconds), so arrivals don't pile up where everyone lands.
const ARRIVED_STAY: (u64, u64) = (2, 6);
const IN_CHAIR: u8 = 2;
const ON_GROUND: u8 = 1;
const GOES_TO_SIT: f32 = 0.08;
const SITS_DOWN: f32 = 0.02;
/// How far it'll walk to a free seat (yalms.)
const SEAT_WITHIN: f32 = 40.0;
/// How near another bot's spot a new one may be (yalms; players' hitboxes are 0.5.)
const APART: f32 = 1.0;
/// Crafters' and gatherers' classes (ClassJob rows 8-18.)
const DOH_DOL: std::ops::RangeInclusive<u8> = 8..=18;
/// How near another bot must be for an emote aimed at it (yalms.)
const EMOTE_AT_WITHIN: f32 = 4.0;

/// Retail's players' jobs in the cities, as a share in tenths of a percent (7,495 players as they came into view.)
/// Crafters and gatherers among them.
const JOBS: [(u8, u16); 43] = [
    (24, 69),
    (21, 60),
    (19, 57),
    (32, 53),
    (25, 49),
    (34, 43),
    (27, 41),
    (37, 40),
    (23, 38),
    (31, 37),
    (38, 35),
    (40, 34),
    (22, 33),
    (41, 33),
    (39, 29),
    (33, 28),
    (35, 28),
    (28, 24),
    (30, 24),
    (20, 22),
    (42, 19),
    (6, 15),
    (16, 13),
    (14, 12),
    (3, 12),
    (8, 12),
    (1, 11),
    (5, 11),
    (4, 10),
    (43, 10),
    (2, 10),
    (7, 10),
    (9, 9),
    (17, 9),
    (18, 9),
    (11, 9),
    (26, 8),
    (15, 8),
    (36, 7),
    (10, 6),
    (12, 6),
    (13, 5),
    (29, 4),
];
/// Their levels: the share (tenths of a percent) in each ten (0 is 1-9) and at 100.
const LEVEL_TENS: [(u8, u16); 11] = [
    (0, 32),
    (1, 36),
    (2, 36),
    (3, 36),
    (4, 24),
    (5, 71),
    (6, 45),
    (7, 51),
    (8, 50),
    (9, 70),
    (10, 550),
];
/// Their online statuses as they came into view (share in the same 7,495), less those a lone player wouldn't show
/// (a party's, Disconnected): none (0), New Adventurer, Away, Mentor, Waiting for the Duty Finder, Trade Mentor,
/// Returner, PvE Mentor, Viewing a Cutscene, Trial Adventurer, PvP Mentor, Camera Mode, Busy, Triple Triad.
const ONLINE_STATUSES: [(u8, u16); 14] = [
    (0, 2405),
    (32, 1703),
    (17, 1172),
    (27, 451),
    (25, 426),
    (29, 264),
    (31, 226),
    (28, 140),
    (15, 95),
    (44, 20),
    (30, 14),
    (18, 14),
    (12, 11),
    (14, 6),
];
/// Two in three wore a title (5,020 of 7,495.)
const TITLE_SHARE: f32 = 0.67;
/// Their idle pose as they came into view (0-6; 4,532 at 0, 400-530 each of the rest.)
const POSES: [(u8, u16); 7] = [
    (0, 4532),
    (1, 436),
    (2, 526),
    (3, 416),
    (4, 512),
    (5, 346),
    (6, 269),
];

/// Retail's Teleport as others see it: a 5 s cast, the player gone 5.9 s after it began (1,433 of Bob's captures),
/// and an arrival's ZoneIn about a second after the player appears (the median of 6,062: 0.9-1.3 s.)
const TELEPORT_ACTION: u32 = 5;
const TELEPORT_CAST: f32 = 5.0;
const TELEPORT_GONE: Duration = Duration::from_millis(5900);
const ZONE_IN_AFTER: Duration = Duration::from_millis(1000);
/// A loading screen's length between zones (a guess; not in the captures, which only see one side of it.)
const LOADING: (u64, u64) = (4, 10);
/// One in eight players who left the capturing player's view teleported out (612 of 4,872); the rest walked out of
/// range, which here is a walk to another of the zone's places, its exits or an inn.
const LEAVES_BY_TELEPORT: f32 = 0.125;
/// After a stay, how often a bot goes somewhere else rather than to another spot by its place (a guess.)
const GOES_ELSEWHERE: f32 = 0.3;
/// Away from its home, how often it goes back after a stay rather than somewhere else again (a guess.)
const GOES_HOME: f32 = 0.6;
/// How long it stays in an inn room, seconds (a guess.)
const INN_STAY: (u64, u64) = (120, 1200);
/// Where it stands round its place when retail's spots say nothing: this far from the aetheryte's crystal, and from a
/// marketboard; and how far a recorded spot may be from them to count as theirs.
const AETHERYTE_RING: (f32, f32) = (4.0, 12.0);
const MARKETBOARD_RING: (f32, f32) = (2.0, 6.0);
const AETHERYTE_SPOTS_WITHIN: f32 = 30.0;
const MARKETBOARD_SPOTS_WITHIN: f32 = 10.0;
/// The actor IDs of world bots' made-up content IDs (party bots' are 0x00B0…, `party_bot::BOT_ID_BASE`.)
pub const CONTENT_ID_BASE: u64 = 0x00B2_0000_0000_0000;

/// A place in a zone: its kind and where it is.
#[derive(Debug, Clone, Default)]
pub struct Places {
    /// Big aetherytes: (Aetheryte row, where the crystal stands, where Teleport lands.)
    pub aetherytes: Vec<(u32, Vec3A, Option<Vec3A>)>,
    /// Aethernet shards (a city's guilds, markets and gates): where each stands.
    pub shards: Vec<Vec3A>,
    pub marketboards: Vec<Vec3A>,
    /// Innkeepers (an ENpc with a Warp into an inn room.)
    pub inns: Vec<Vec3A>,
    /// Exits: where, the territory they lead to, and its pop range there.
    pub exits: Vec<(Vec3A, u16, u32)>,
    /// The zone's pop ranges (where an exit from elsewhere leaves you.)
    pub pops: HashMap<u32, Vec3A>,
}

impl Places {
    /// A zone's places, from its layout and the game's data: big aetherytes and shards (Aetheryte layout entries,
    /// their row's IsAetheryte), marketboards (event objects whose handler is a CustomTalk named `CmnDefMarketBoard…`),
    /// innkeepers (ENpcs with a Warp into a territory meant for an inn) and exits (ExitRange entries into another
    /// territory.)
    pub fn of(zone: &Zone, game_data: &mut GameData, aetherytes: &[AetheryteRowPlace]) -> Self {
        let mut places = Places {
            pops: zone
                .cached_pop_ranges
                .iter()
                .map(|(id, (position, _))| (*id, position.0))
                .collect(),
            ..Default::default()
        };
        for object in zone.layout_objects() {
            let (_, _, translation) =
                Affine3A::from(object.transform).to_scale_rotation_translation();
            let at: Vec3A = translation.into();
            match &object.data {
                LayerEntryData::Aetheryte(aetheryte) => {
                    let row = aetheryte.parent_data.base_id;
                    match aetherytes.iter().find(|place| place.id == row) {
                        Some(place) if place.big => {
                            let landing = place
                                .levels
                                .iter()
                                .find_map(|level| places.pops.get(level).copied());
                            places.aetherytes.push((row, at, landing));
                        }
                        Some(_) => places.shards.push(at),
                        None => {}
                    }
                }
                LayerEntryData::EventObject(eobj) => {
                    let data = game_data
                        .eobj_sheet
                        .row(eobj.parent_data.base_id)
                        .map(|row| row.Data)
                        .unwrap_or_default();
                    if data >> 16 == HANDLER_CUSTOM_TALK
                        && game_data
                            .custom_talk_script(data)
                            .is_some_and(|name| name.starts_with("CmnDefMarketBoard"))
                    {
                        places.marketboards.push(at);
                    }
                }
                LayerEntryData::EventNPC(npc) => {
                    let base = npc.parent_data.parent_data.base_id;
                    let innkeeper = game_data.enpc_handlers(base).into_iter().any(|handler| {
                        handler >> 16 == HANDLER_WARP
                            && game_data.get_warp(handler).is_some_and(|(_, territory)| {
                                game_data.get_intended_use(territory as u32)
                                    == Some(TerritoryIntendedUse::Inn)
                            })
                    });
                    if innkeeper {
                        places.inns.push(at);
                    }
                }
                LayerEntryData::ExitRange(exit)
                    if exit.territory_type != 0 && exit.territory_type != zone.id =>
                {
                    places
                        .exits
                        .push((at, exit.territory_type, exit.destination_instance_id));
                }
                _ => {}
            }
        }
        places
    }

    /// Where someone arriving in the zone stands: its first big aetheryte's landing, else that aetheryte, else its
    /// first pop range. (A spot of a bot's own must be walkable from here: `Ground::own`.)
    pub fn landing(&self) -> Option<Vec3A> {
        self.aetherytes
            .iter()
            .find_map(|(_, _, landing)| *landing)
            .or_else(|| self.aetherytes.first().map(|(_, at, _)| *at))
            .or_else(|| {
                self.pops
                    .iter()
                    .min_by_key(|(id, _)| **id)
                    .map(|(_, at)| *at)
            })
    }
}

/// Handler types (`HandlerType`): a Warp, a CustomTalk.
const HANDLER_WARP: u32 = 2;
const HANDLER_CUSTOM_TALK: u32 = 11;

/// What a bot keeps coming back to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    /// The big aetheryte of this Aetheryte row.
    Aetheryte(u32),
    /// Its zone's marketboard of this index (`Places::marketboards`.)
    Marketboard(usize),
}

/// What it's doing.
#[derive(Debug, Clone, PartialEq)]
enum Doing {
    /// Standing, until its stay is over.
    Standing,
    /// Walking to `to`, then what follows.
    Walking { to: Vec3A, then: Then },
    /// Casting Teleport to this Aetheryte row's zone; gone at `until`.
    Teleporting { to: u16, aetheryte: u32 },
    /// Between zones (a loading screen), arriving at `until`.
    Away { to: u16, arrive: Arrive },
    /// In an inn room, coming out at `until` where it went in.
    Inside { at: Vec3A },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Then {
    Stand,
    /// Out through an exit into this territory, at this pop range.
    Exit {
        to: u16,
        pop: u32,
    },
    /// Into an inn room.
    Inn,
    /// Into this seat of its zone (`seats`), to sit.
    Sit(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Arrive {
    /// By Teleport, beside this Aetheryte row's crystal.
    Aetheryte(u32),
    /// Through an exit, at this pop range.
    Pop(u32),
}

/// Who a bot is: what it looks like to others.
#[derive(Debug, Clone)]
pub struct Who {
    pub name: String,
    pub classjob: u8,
    pub level: u8,
    /// (Its race and the rest, for stage 2's gear: `docs/src/fork/world-bots.md`.)
    #[allow(dead_code)]
    pub look: CustomizeData,
    /// Its spawn as an NPC (position and the rest set as it's shown), and its HP.
    pub spawn: SpawnNpc,
    pub online_status: OnlineStatus,
    pub title_id: u16,
    pub pose: u8,
    /// What it wears, for Examine, and its item level.
    pub equipped: EquippedStorage,
    pub item_level: u16,
    /// The server's glamour it wears, by name (kept if it's befriended.)
    pub glamour: Option<String>,
    /// Its made-up content ID (`CONTENT_ID_BASE` and a random number), the same each time it's shown: the friend
    /// list's and the party list's, so a friend found in the crowd is the same one.
    pub content_id: u64,
}

/// One world bot.
#[derive(Debug, Clone)]
pub struct WorldBot {
    pub who: Who,
    /// Its home: the territory and the place there it keeps coming back to.
    pub home: (u16, Anchor),
    /// The territory it's in (or going to, while away.)
    pub at: u16,
    /// Where it stands or is walking from; None until it's been put somewhere in a zone whose places are known.
    pub spot: Option<Vec3A>,
    pub facing: f32,
    doing: Doing,
    /// When what it's doing is over.
    until: Instant,
    /// Its actor, while it's shown in its zone's instance.
    pub actor: Option<ObjectId>,
    /// Its walk, while shown.
    path: VecDeque<Vec3A>,
    /// When to play its arrival (ZoneIn) to those who see it.
    zone_in_at: Option<Instant>,
    /// Whether its Teleport cast has been shown.
    cast_shown: bool,
    /// Sitting (SetMode 11's argument: in a chair, or on the ground), and in which of its zone's seats.
    sitting: Option<u8>,
    seat: Option<usize>,
    /// It joined a player's party (as a party bot beside them): Teleporting away, then someone new.
    joined: bool,
    /// Just arrived where everyone lands (an aetheryte's landing, an exit's pop range): it walks off to a spot next.
    arrived: bool,
}

impl WorldBot {
    /// Whether it's somewhere in its zone to be seen.
    fn visible(&self) -> bool {
        matches!(
            self.doing,
            Doing::Standing | Doing::Walking { .. } | Doing::Teleporting { .. }
        )
    }
}

/// Everything about the world's bots, kept by the world server.
#[derive(Debug, Default)]
pub struct WorldBots {
    /// The settings as last read, and the file's modification time then.
    settings: Option<(Option<SystemTime>, Settings)>,
    pub bots: Vec<WorldBot>,
    /// Each territory's places, read from its zone the first time an instance of it is seen.
    pub places: HashMap<u16, Arc<Places>>,
    /// The game's data, read once: Aetheryte rows, titles, one-shot emotes, territories' intended use.
    aetherytes: Vec<AetheryteRowPlace>,
    titles: Vec<u16>,
    emotes: Vec<(u32, u32, u8)>,
    ready: bool,
    /// Territories whose marketboards have had their bots made.
    markets_made: HashSet<u16>,
    /// When the settings were last checked.
    checked: Option<Instant>,
}

/// Retail's recorded spots by territory (`resources/data/world_bot_spots.json`): [x, y, z, seconds, players].
fn recorded_spots() -> &'static HashMap<u16, Vec<[f32; 5]>> {
    static SPOTS: std::sync::OnceLock<HashMap<u16, Vec<[f32; 5]>>> = std::sync::OnceLock::new();
    SPOTS.get_or_init(|| {
        let value: Value = serde_json::from_str(include_str!(
            "../../../../resources/data/world_bot_spots.json"
        ))
        .unwrap_or_default();
        value
            .get("territories")
            .and_then(Value::as_object)
            .map(|territories| {
                territories
                    .iter()
                    .filter_map(|(territory, cells)| {
                        let cells = cells
                            .as_array()?
                            .iter()
                            .filter_map(|cell| {
                                let cell: Vec<f32> = cell
                                    .as_array()?
                                    .iter()
                                    .filter_map(|v| v.as_f64().map(|v| v as f32))
                                    .collect();
                                (cell.len() == 5)
                                    .then(|| [cell[0], cell[1], cell[2], cell[3], cell[4]])
                            })
                            .collect();
                        Some((territory.parse().ok()?, cells))
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// Retail's chairs and benches by territory (`resources/data/world_bot_seats.json`): [x, y, z, facing.]
fn seats() -> &'static HashMap<u16, Vec<[f32; 4]>> {
    static SEATS: std::sync::OnceLock<HashMap<u16, Vec<[f32; 4]>>> = std::sync::OnceLock::new();
    SEATS.get_or_init(|| {
        let value: Value = serde_json::from_str(include_str!(
            "../../../../resources/data/world_bot_seats.json"
        ))
        .unwrap_or_default();
        value
            .get("seats")
            .and_then(Value::as_object)
            .map(|territories| {
                territories
                    .iter()
                    .filter_map(|(territory, seats)| {
                        let seats = seats
                            .as_array()?
                            .iter()
                            .filter_map(|seat| {
                                let seat: Vec<f32> = seat
                                    .as_array()?
                                    .iter()
                                    .filter_map(|v| v.as_f64().map(|v| v as f32))
                                    .collect();
                                (seat.len() == 4).then(|| [seat[0], seat[1], seat[2], seat[3]])
                            })
                            .collect();
                        Some((territory.parse().ok()?, seats))
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// One of `choices` by weight.
fn weighted<T: Copy>(choices: &[(T, u16)]) -> Option<T> {
    let total: u32 = choices.iter().map(|(_, weight)| *weight as u32).sum();
    if total == 0 {
        return None;
    }
    let mut roll = fastrand::u32(..total);
    for (choice, weight) in choices {
        if roll < *weight as u32 {
            return Some(*choice);
        }
        roll -= *weight as u32;
    }
    None
}

/// A stay's length, drawn from retail's percentiles for the zone (straight lines between them, the ends stretched.)
pub fn stay_length(territory: u16) -> Duration {
    let table = STAY_SECONDS
        .iter()
        .find(|(t, _)| *t == territory)
        .map(|(_, table)| *table)
        .unwrap_or(STAY_ELSEWHERE);
    let points = [
        (0.0, 8.0),
        (0.1, table[0]),
        (0.25, table[1]),
        (0.5, table[2]),
        (0.75, table[3]),
        (0.9, table[4]),
        (1.0, table[4] * 2.0),
    ];
    let roll = fastrand::f32();
    let seconds = points
        .windows(2)
        .find(|pair| roll <= pair[1].0)
        .map(|pair| {
            let ((p0, s0), (p1, s1)) = (pair[0], pair[1]);
            s0 + (s1 - s0) * (roll - p0) / (p1 - p0)
        })
        .unwrap_or(table[4]);
    Duration::from_secs_f32(seconds)
}

/// A level drawn from retail's city players' (`LEVEL_TENS`.)
fn random_level() -> u8 {
    match weighted(&LEVEL_TENS).unwrap_or(10) {
        10 => 100,
        0 => fastrand::u8(1..=9),
        ten => ten * 10 + fastrand::u8(0..=9),
    }
}

/// An online status as retail's players showed (`ONLINE_STATUSES`), fitting its level: the New Adventurer's sprout
/// only under 60, the mentors' crowns only from 80 (a guess at what's likely; the captures don't say per level.)
fn random_online_status(level: u8) -> OnlineStatus {
    let status = weighted(&ONLINE_STATUSES).unwrap_or(0);
    let status = match status {
        32 if level >= 60 => 0,
        27..=30 if level < 80 => 0,
        other => other,
    };
    OnlineStatus::from_repr(status).unwrap_or(OnlineStatus::Offline)
}

impl WorldBots {
    /// The settings, read again when the file has changed (checked every few seconds.) Returns whether they changed.
    fn refresh_settings(&mut self, now: Instant) -> bool {
        if self
            .checked
            .is_some_and(|at| now.duration_since(at) < Duration::from_secs(3))
        {
            return false;
        }
        self.checked = Some(now);
        let path = settings_path();
        let modified = std::fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .ok();
        if self
            .settings
            .as_ref()
            .is_some_and(|(at, _)| *at == modified)
        {
            return false;
        }
        let settings = match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => Settings::from_json(&value),
                Err(err) => {
                    tracing::warn!("World bots: {} isn't JSON ({err})", path.display());
                    Settings::default()
                }
            },
            Err(_) => Settings::default(),
        };
        let changed = self
            .settings
            .as_ref()
            .is_none_or(|(_, old)| *old != settings);
        if changed {
            tracing::info!(
                "World bots: {} from {} ({} cities, {} per other aetheryte, {}-{} per marketboard, travel {})",
                if settings.enabled { "on" } else { "off" },
                path.display(),
                settings.cities.len(),
                settings.per_aetheryte,
                settings.per_marketboard.0,
                settings.per_marketboard.1,
                settings.travel
            );
        }
        self.settings = Some((modified, settings));
        changed
    }

    fn settings(&self) -> Settings {
        self.settings
            .as_ref()
            .map(|(_, settings)| settings.clone())
            .unwrap_or_default()
    }

    /// The game's lists, read once.
    fn read_game_data(&mut self, game_data: &mut GameData) {
        if self.ready {
            return;
        }
        self.ready = true;
        self.aetherytes = game_data.aetheryte_places();
        // Only the big ones in towns and the open world gather bots.
        self.aetherytes.retain(|place| {
            !place.big
                || matches!(
                    game_data.get_intended_use(place.territory as u32),
                    Some(TerritoryIntendedUse::Town | TerritoryIntendedUse::Overworld)
                )
        });
        self.titles = game_data.title_ids();
        let ids: Vec<u32> = EMOTES.iter().map(|(id, ..)| *id).collect();
        let one_shot = game_data.one_shot_emotes(&ids);
        self.emotes = EMOTES
            .iter()
            .filter(|(id, ..)| one_shot.contains(id))
            .copied()
            .collect();
        tracing::info!(
            "World bots: {} big aetherytes in towns and the open world, {} titles, {} emotes",
            self.aetherytes.iter().filter(|place| place.big).count(),
            self.titles.len(),
            self.emotes.len()
        );
    }

    /// A new bot's look, class, level, gear, glamour, status and title.
    fn make_who(&self, game_data: &mut GameData) -> Option<Who> {
        let classjob = weighted(&JOBS.map(|(job, share)| (job, share)))?;
        let level = random_level();
        let look = party_bot::random_look(game_data);
        let (_, parent, _) = game_data
            .classjob_role(classjob)
            .unwrap_or((0, classjob, 0));
        let wearable = game_data.wearable_restrictions(look.race, look.gender);
        let gear = game_data.best_gear(classjob, level, &wearable);
        let mut wearing = party_bot::wear_gear(game_data, &gear);
        let glamours = glamour::glamours(game_data);
        let glamour = glamour::pick(&glamours, look.gender, classjob, parent).map(|chosen| {
            party_bot::wear_glamour(game_data, &mut wearing, chosen, &wearable);
            chosen.name.clone()
        });
        let item_level = wearing.equipped.calculate_item_level(game_data);
        let item_ids: Vec<u32> = gear.iter().map(|(_, id)| *id).collect();
        let weapon = gear
            .iter()
            .find(|(slot, _)| *slot == party_bot::MAIN_HAND)
            .map(|(_, id)| *id);
        let hp = party_bot::bot_parameters(
            game_data,
            classjob,
            look.tribe as u8,
            level,
            &item_ids,
            weapon,
        )
        .map(|parameters| parameters.hp.max(1))
        .unwrap_or(1000);
        let head_model = wearing.legacy_model_ids(game_data)[0].id;
        let hide_head =
            head_model != 0 && !party_bot::race_has_head_model(game_data, &look, head_model);
        let name = party_bot::random_name();
        let spawn = SpawnNpc {
            common: CommonSpawn {
                main_weapon_model: wearing.get_main_weapon_id(game_data),
                // Retail's idle crafters and gatherers show only their main tool: no off-hand, no station
                // (Bob's captures: the spawns of players of classes 8-18 standing about in the cities.)
                sec_weapon_model: if DOH_DOL.contains(&classjob) {
                    Default::default()
                } else {
                    wearing.get_sub_weapon_id(game_data)
                },
                models: wearing.legacy_model_ids(game_data),
                second_model_stain_ids: wearing.second_model_stain_ids(),
                max_health_points: hp,
                health_points: hp,
                max_resource_points: 10000,
                resource_points: 10000,
                object_kind: ObjectKind::BattleNpc(BattleNpcSubKind::Chocobo),
                display_flags: if hide_head {
                    DisplayFlag::HIDE_HEAD
                } else {
                    DisplayFlag::empty()
                },
                battalion: 0,
                model_chara: 0,
                level,
                class_job: classjob,
                name: name.clone(),
                look: look.clone(),
                target_id: ObjectTypeId::default(),
                ..Default::default()
            },
            character_data_flags: CharacterDataFlag::from_bits_retain(0x1),
            ..Default::default()
        };
        let title_id = if !self.titles.is_empty() && fastrand::f32() < TITLE_SHARE {
            self.titles[fastrand::usize(..self.titles.len())]
        } else {
            0
        };
        Some(Who {
            name,
            classjob,
            level,
            look,
            spawn,
            online_status: random_online_status(level),
            title_id,
            pose: weighted(&POSES).unwrap_or(0),
            equipped: wearing.equipped,
            item_level,
            glamour,
            content_id: CONTENT_ID_BASE | fastrand::u32(1..) as u64,
        })
    }

    /// A bot at home by `anchor` in `territory`, standing (its spot is chosen when it's first shown.)
    fn new_bot(
        &self,
        game_data: &mut GameData,
        territory: u16,
        anchor: Anchor,
        now: Instant,
    ) -> Option<WorldBot> {
        let who = self.make_who(game_data)?;
        Some(WorldBot {
            who,
            home: (territory, anchor),
            at: territory,
            spot: None,
            facing: fastrand::f32() * std::f32::consts::TAU - std::f32::consts::PI,
            doing: Doing::Standing,
            // Staggered, so they don't all move at once.
            until: now + stay_length(territory).mul_f32(fastrand::f32()),
            actor: None,
            path: VecDeque::new(),
            zone_in_at: None,
            cast_shown: false,
            sitting: None,
            seat: None,
            joined: false,
            arrived: false,
        })
    }

    /// The population for the settings: bots round each city's big aetheryte, and round every other big one in towns
    /// and the open world. (Marketboards' come when their zone's places are first known: `make_market_bots`.)
    fn populate(&mut self, game_data: &mut GameData, now: Instant) {
        let settings = self.settings();
        self.bots.clear();
        self.markets_made.clear();
        if !settings.enabled {
            return;
        }
        let big: Vec<(u32, u16)> = self
            .aetherytes
            .iter()
            .filter(|place| place.big)
            .map(|place| (place.id, place.territory))
            .collect();
        let mut made = Vec::new();
        for (territory, count) in &settings.cities {
            let Some((row, _)) = big.iter().find(|(_, t)| t == territory) else {
                tracing::warn!("World bots: no big aetheryte in territory {territory}");
                continue;
            };
            for _ in 0..*count {
                made.extend(self.new_bot(game_data, *territory, Anchor::Aetheryte(*row), now));
            }
        }
        for (row, territory) in &big {
            if settings.cities.iter().any(|(t, _)| t == territory) {
                continue;
            }
            for _ in 0..settings.per_aetheryte {
                made.extend(self.new_bot(game_data, *territory, Anchor::Aetheryte(*row), now));
            }
        }
        tracing::info!("World bots: {} made", made.len());
        self.bots = made;
    }

    /// Bots round a zone's marketboards, once its places are known.
    fn make_market_bots(&mut self, game_data: &mut GameData, territory: u16, now: Instant) {
        if self.markets_made.contains(&territory) {
            return;
        }
        let Some(places) = self.places.get(&territory).cloned() else {
            return;
        };
        self.markets_made.insert(territory);
        let (low, high) = self.settings().per_marketboard;
        let mut made = 0;
        for index in 0..places.marketboards.len() {
            for _ in 0..fastrand::u32(low..=high) {
                if let Some(bot) =
                    self.new_bot(game_data, territory, Anchor::Marketboard(index), now)
                {
                    self.bots.push(bot);
                    made += 1;
                }
            }
        }
        if made > 0 {
            tracing::info!(
                "World bots: {made} at the {} marketboards of territory {territory}",
                places.marketboards.len()
            );
        }
    }
}

/// Where its place is in `places`, and how far round it it stands.
fn anchor_point(places: &Places, anchor: Anchor) -> Option<(Vec3A, (f32, f32), f32)> {
    match anchor {
        Anchor::Aetheryte(row) => places
            .aetherytes
            .iter()
            .find(|(id, ..)| *id == row)
            .or_else(|| places.aetherytes.first())
            .map(|(_, at, _)| (*at, AETHERYTE_RING, AETHERYTE_SPOTS_WITHIN)),
        Anchor::Marketboard(index) => places
            .marketboards
            .get(index)
            .map(|at| (*at, MARKETBOARD_RING, MARKETBOARD_SPOTS_WITHIN)),
    }
}

/// What a spot is stood on: the zone's navmesh and collision, when it's shown.
#[derive(Clone, Copy, Default)]
struct Ground<'a> {
    navmesh: Option<&'a crate::navmesh::Navmesh>,
    collision: Option<&'a crate::collision::Collision>,
}

/// How far a spot not recorded may be moved onto the navmesh, and how near its path from the zone's landing must end
/// to it (yalms.)
const ON_MESH_WITHIN: f32 = 1.0;
/// How near a recorded spot's height the collision's floor must be to be taken over it (yalms.)
const RECORDED_FLOOR_WITHIN: f32 = 0.25;

impl<'a> Ground<'a> {
    fn of(instance: &'a Instance) -> Self {
        Ground {
            navmesh: Some(&instance.navmesh),
            collision: instance.collision.as_deref(),
        }
    }

    fn navmesh(&self) -> Option<&'a crate::navmesh::Navmesh> {
        self.navmesh.filter(|navmesh| navmesh.is_available())
    }

    /// The collision's floor at (x, z) within `within` of `y`.
    fn floor_near(&self, at: Vec3A, within: f32) -> Option<f32> {
        self.collision?
            .floor(at.x, at.z, at.y + within, at.y - within)
            .filter(|y| (y - at.y).abs() <= within)
    }

    /// A recorded spot (retail's players stood there, at that height): on the collision's floor only when it's within
    /// a quarter yalm of that height, else as recorded. The navmesh and the collision can miss what was stood on: New
    /// Gridania's bridge by the aetheryte plaza, where retail's players stood at 1.28 and Kawari put bots at -0.13 and
    /// then 0.30, under it (Bob's bundles 20261111-082127 and -091633.)
    fn recorded(&self, spot: Vec3A) -> Vec3A {
        match self.floor_near(spot, RECORDED_FLOOR_WITHIN) {
            Some(y) => Vec3A::new(spot.x, y, spot.z),
            None => spot,
        }
    }

    /// A spot of its own (a ring round a place): the navmesh's nearest point within a yalm, which a walker from
    /// `from` (the zone's landing) gets to, so it's never in a wall or behind a marketboard (Bob's bundle
    /// 20261111-091633); on the collision's floor there. Without a navmesh, the collision's floor near it, or as it is.
    fn own(&self, spot: Vec3A, from: Option<Vec3A>) -> Option<Vec3A> {
        let Some(navmesh) = self.navmesh() else {
            return Some(match self.floor_near(spot, 3.0) {
                Some(y) => Vec3A::new(spot.x, y, spot.z),
                None => spot,
            });
        };
        let point = navmesh.nearest_point(spot, [ON_MESH_WITHIN, 3.0, ON_MESH_WITHIN])?;
        if let Some(from) = from {
            let path = navmesh.calculate_path(from, point, f32::INFINITY);
            let reached = path.last().is_some_and(|end| {
                Vec3A::new(end.x - point.x, 0.0, end.z - point.z).length() <= ON_MESH_WITHIN
            });
            if !reached && from.distance(point) > ON_MESH_WITHIN {
                return None;
            }
        }
        Some(match self.floor_near(point, 0.5) {
            Some(y) => Vec3A::new(point.x, y, point.z),
            None => point,
        })
    }
}

/// Where the other bots of `territory` stand, or are walking to (so none stands in another.)
fn taken_spots(bots: &[WorldBot], territory: u16, except: usize) -> Vec<Vec3A> {
    bots.iter()
        .enumerate()
        .filter(|(index, bot)| *index != except && bot.at == territory && bot.visible())
        .filter_map(|(_, bot)| match &bot.doing {
            Doing::Walking { to, .. } => Some(*to),
            _ => bot.spot,
        })
        .collect()
}

/// The seats of `territory` other bots sit in or are walking to.
fn taken_seats(bots: &[WorldBot], territory: u16, except: usize) -> HashSet<usize> {
    bots.iter()
        .enumerate()
        .filter(|(index, bot)| *index != except && bot.at == territory)
        .filter_map(|(_, bot)| match &bot.doing {
            Doing::Walking {
                then: Then::Sit(seat),
                ..
            } => Some(*seat),
            _ => bot.seat,
        })
        .collect()
}

/// A spot to stand by `anchor`: one of retail's recorded spots near it (weighted by how long players stood there, a
/// little apart; `Ground::recorded`), or else somewhere in a ring round it that a walker from the zone's landing gets
/// to (`Ground::own`), not within `APART` of the `taken` spots. Failing those, where Teleport lands by its aetheryte, or
/// the zone's landing (never the place itself: a marketboard's middle is inside it.)
fn spot_by(
    territory: u16,
    places: &Places,
    anchor: Anchor,
    ground: Ground,
    taken: &[Vec3A],
) -> Option<Vec3A> {
    let (center, ring, within) = anchor_point(places, anchor)?;
    let from = places.landing();
    let recorded: Vec<([f32; 5], u16)> = recorded_spots()
        .get(&territory)
        .map(|cells| {
            cells
                .iter()
                .filter(|cell| Vec3A::new(cell[0], cell[1], cell[2]).distance(center) <= within)
                .map(|cell| (*cell, (cell[3] / 60.0).clamp(1.0, 60000.0) as u16))
                .collect()
        })
        .unwrap_or_default();
    for attempt in 0..24 {
        let spot = match weighted(&recorded) {
            Some(cell) => Some(ground.recorded(
                Vec3A::new(cell[0], cell[1], cell[2])
                    + Vec3A::new(fastrand::f32() - 0.5, 0.0, fastrand::f32() - 0.5) * 1.6,
            )),
            None => {
                // Further out the more it's tried (a small place hemmed in by walls.)
                let wider = 1.0 + attempt as f32 / 12.0;
                let angle = fastrand::f32() * std::f32::consts::TAU;
                let distance = (ring.0 + fastrand::f32() * (ring.1 - ring.0)) * wider;
                ground.own(
                    center + Vec3A::new(angle.sin(), 0.0, angle.cos()) * distance,
                    from,
                )
            }
        };
        if let Some(spot) = spot
            && taken.iter().all(|other| other.distance(spot) >= APART)
        {
            return Some(spot);
        }
    }
    match anchor {
        Anchor::Aetheryte(row) => places
            .aetherytes
            .iter()
            .find(|(id, ..)| *id == row)
            .and_then(|(_, _, landing)| *landing),
        Anchor::Marketboard(_) => None,
    }
    .or(from)
    .or(Some(center))
}

/// A free seat of `territory` within `SEAT_WITHIN` of `here`, if any.
fn free_seat(territory: u16, here: Vec3A, taken: &HashSet<usize>) -> Option<(usize, [f32; 4])> {
    let free: Vec<(usize, [f32; 4])> = seats()
        .get(&territory)?
        .iter()
        .enumerate()
        .filter(|(index, seat)| {
            !taken.contains(index)
                && Vec3A::new(seat[0], seat[1], seat[2]).distance(here) <= SEAT_WITHIN
        })
        .map(|(index, seat)| (index, *seat))
        .collect();
    (!free.is_empty()).then(|| free[fastrand::usize(..free.len())])
}

/// Where to stand up to a seat: retail's players got up 0.42 yalms ahead of it, half a yalm down.
fn before_seat([x, y, z, facing]: [f32; 4]) -> Vec3A {
    Vec3A::new(
        x + facing.sin() * SEAT_STAND_AHEAD,
        y - SEAT_STAND_DOWN,
        z + facing.cos() * SEAT_STAND_AHEAD,
    )
}

/// Where to walk to, going to a place of its zone: a spot of its own 2-5 yalms from a shard, marketboard, aetheryte or
/// innkeeper (on the navmesh, reached from the zone's landing, apart from the others: never in the place or behind a
/// counter); an exit's own spot, or the navmesh's nearest point within 3 yalms of it.
fn near_place(
    ground: Ground,
    at: Vec3A,
    then: Then,
    from: Option<Vec3A>,
    taken: &[Vec3A],
) -> Option<Vec3A> {
    if let Then::Exit { .. } = then {
        return ground
            .own(at, from)
            .or_else(|| ground.navmesh()?.nearest_point(at, [3.0, 3.0, 3.0]))
            .or(Some(at));
    }
    for _ in 0..8 {
        let angle = fastrand::f32() * std::f32::consts::TAU;
        let distance = 2.0 + fastrand::f32() * 3.0;
        if let Some(spot) = ground.own(
            at + Vec3A::new(angle.sin(), 0.0, angle.cos()) * distance,
            from,
        ) && taken.iter().all(|other| other.distance(spot) >= APART)
        {
            return Some(spot);
        }
    }
    None
}

/// Somewhere to go in its zone other than its place: another place of the zone (a shard, a marketboard, another big
/// aetheryte), an inn or an exit.
fn trip_in_zone(places: &Places, here: Vec3A) -> Option<(Vec3A, Then)> {
    let mut choices: Vec<((Vec3A, Then), u16)> = Vec::new();
    for at in places
        .shards
        .iter()
        .chain(places.marketboards.iter())
        .chain(places.aetherytes.iter().map(|(_, at, _)| at))
    {
        if at.distance(here) > 15.0 {
            choices.push(((*at, Then::Stand), 4));
        }
    }
    for at in &places.inns {
        choices.push(((*at, Then::Inn), 6));
    }
    for (at, to, pop) in &places.exits {
        choices.push(((*at, Then::Exit { to: *to, pop: *pop }), 3));
    }
    let (to, then) = weighted(&choices)?;
    // Near a place, not on it: players stood round a shard or a board, not in it.
    let angle = fastrand::f32() * std::f32::consts::TAU;
    let offset = if then == Then::Stand {
        Vec3A::new(angle.sin(), 0.0, angle.cos()) * (2.0 + fastrand::f32() * 3.0)
    } else {
        Vec3A::ZERO
    };
    Some((to + offset, then))
}

/// The big aetheryte Teleport takes a bot to, elsewhere: the cities' more often (three in four), any other big one in
/// a town or the open world otherwise.
fn teleport_destination(bots: &WorldBots, from: u16, settings: &Settings) -> Option<(u16, u32)> {
    let big: Vec<&AetheryteRowPlace> = bots
        .aetherytes
        .iter()
        .filter(|place| place.big && place.territory != from)
        .collect();
    let cities: Vec<&&AetheryteRowPlace> = big
        .iter()
        .filter(|place| settings.cities.iter().any(|(t, _)| *t == place.territory))
        .collect();
    let place = if !cities.is_empty() && fastrand::f32() < 0.75 {
        **cities.get(fastrand::usize(..cities.len()))?
    } else {
        *big.get(fastrand::usize(..big.len().max(1)))?
    };
    Some((place.territory, place.id))
}

/// Who's where: each public instance with players in it, by territory (index into `instances`.)
fn shown_zones(instances: &[Instance], game_data: &mut GameData) -> HashMap<u16, usize> {
    let mut zones = HashMap::new();
    for (index, instance) in instances.iter().enumerate() {
        if instance.content_finder_condition_id != 0 {
            continue;
        }
        if !matches!(
            game_data.get_intended_use(instance.zone.id as u32),
            Some(TerritoryIntendedUse::Town | TerritoryIntendedUse::Overworld)
        ) {
            continue;
        }
        if instance
            .actors
            .values()
            .any(|actor| matches!(actor, NetworkedActor::Player { .. }))
        {
            zones.insert(instance.zone.id, index);
        }
    }
    zones
}

/// The world's bots, each server tick: the settings, the population, their decisions and walks, and showing them in
/// the instances players are in.
pub fn update(
    data: &mut WorldServer,
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
) {
    let now = Instant::now();
    let WorldServer {
        instances,
        world_bots,
        ..
    } = data;
    let changed = world_bots.refresh_settings(now);
    let settings = world_bots.settings();
    if changed {
        // Everyone shown goes, and a new population comes (or none.)
        for bot in &world_bots.bots {
            if let Some(actor) = bot.actor {
                for instance in instances.iter_mut() {
                    if instance.world_bots.remove(&actor) {
                        network.lock().remove_actor(instance, actor);
                    }
                }
            }
        }
        if settings.enabled {
            let mut game_data = game_data.lock();
            world_bots.read_game_data(&mut game_data);
            world_bots.populate(&mut game_data, now);
        } else {
            world_bots.bots.clear();
        }
    }
    if !settings.enabled {
        return;
    }

    let zones = {
        let mut game_data = game_data.lock();
        let zones = shown_zones(instances, &mut game_data);
        // A zone's places, the first time it's seen; its marketboards' bots then.
        for (territory, index) in &zones {
            if !world_bots.places.contains_key(territory) {
                let places = Places::of(
                    &instances[*index].zone,
                    &mut game_data,
                    &world_bots.aetherytes,
                );
                tracing::info!(
                    "World bots: territory {territory}: {} big aetherytes, {} shards, {} marketboards, {} innkeepers, {} exits",
                    places.aetherytes.len(),
                    places.shards.len(),
                    places.marketboards.len(),
                    places.inns.len(),
                    places.exits.len()
                );
                world_bots.places.insert(*territory, Arc::new(places));
            }
            world_bots.make_market_bots(&mut game_data, *territory, now);
        }
        zones
    };

    // Shown bots whose instance has gone (everyone left) are hidden again.
    for bot in &mut world_bots.bots {
        if let Some(actor) = bot.actor {
            let still = zones
                .get(&bot.at)
                .is_some_and(|index| instances[*index].world_bots.contains(&actor));
            if !still {
                bot.actor = None;
                bot.path.clear();
            }
        }
    }

    // Each bot: its time, and what comes next.
    let count = world_bots.bots.len();
    for index in 0..count {
        let shown_in = zones.get(&world_bots.bots[index].at).copied();
        step(
            world_bots,
            index,
            shown_in.map(|i| &mut instances[i]),
            network,
            Some(game_data),
            &settings,
            now,
        );
    }
}

/// One bot's tick.
fn step(
    world_bots: &mut WorldBots,
    index: usize,
    mut instance: Option<&mut Instance>,
    network: &Arc<Mutex<NetworkState>>,
    game_data: Option<&Arc<Mutex<GameData>>>,
    settings: &Settings,
    now: Instant,
) {
    let places = world_bots.places.get(&world_bots.bots[index].at).cloned();
    let taken = if world_bots.bots[index].spot.is_none() && instance.is_some() {
        taken_spots(&world_bots.bots, world_bots.bots[index].at, index)
    } else {
        Vec::new()
    };
    let bot = &mut world_bots.bots[index];

    // Shown: put in its zone's instance if it's to be seen and isn't, taken out if it isn't to be.
    if let Some(instance) = instance.as_deref_mut() {
        if bot.visible() && bot.actor.is_none() {
            if bot.spot.is_none()
                && let Some(places) = &places
            {
                let anchor = if bot.home.0 == bot.at {
                    bot.home.1
                } else {
                    Anchor::Aetheryte(places.aetherytes.first().map(|(id, ..)| *id).unwrap_or(0))
                };
                bot.spot = spot_by(bot.at, places, anchor, Ground::of(instance), &taken);
            }
            if let (Some(spot), Some(game_data)) = (bot.spot, game_data) {
                show(bot, instance, network, game_data, spot);
            }
        } else if !bot.visible()
            && let Some(actor) = bot.actor.take()
        {
            instance.world_bots.remove(&actor);
            network.lock().remove_actor(instance, actor);
        }
    }

    // Its arrival, a second after it appeared.
    if let (Some(at), Some(actor), Some(instance)) =
        (bot.zone_in_at, bot.actor, instance.as_deref_mut())
        && now >= at
    {
        bot.zone_in_at = None;
        network.lock().send_ac_in_range_inclusive_instance(
            instance,
            actor,
            ActorControlCategory::ZoneIn {
                warp_finish_anim: 1,
                raise_anim: 0,
                unk1: 0,
            },
        );
    }

    match bot.doing.clone() {
        Doing::Standing => {
            if let (Some(actor), Some(instance)) = (bot.actor, instance.as_deref_mut()) {
                idle(bot, actor, instance, network, &world_bots.emotes);
            }
            if now >= bot.until {
                decide(
                    world_bots,
                    index,
                    places.as_deref(),
                    instance,
                    network,
                    settings,
                    now,
                );
            }
        }
        Doing::Walking { to, then } => {
            let arrived = match (bot.actor, instance.as_deref_mut()) {
                (Some(actor), Some(instance)) => walk(bot, actor, instance, network, to),
                _ => now >= bot.until,
            };
            if arrived {
                bot.spot = Some(to);
                bot.path.clear();
                match then {
                    Then::Stand => {
                        bot.doing = Doing::Standing;
                        bot.until = now + stay_length(bot.at);
                    }
                    Then::Sit(seat) => {
                        if let Some([x, y, z, facing]) = seats()
                            .get(&bot.at)
                            .and_then(|seats| seats.get(seat))
                            .copied()
                        {
                            bot.spot = Some(Vec3A::new(x, y, z));
                            bot.facing = facing;
                            sit(bot, instance.as_deref_mut(), network, IN_CHAIR, Some(seat));
                        }
                        bot.doing = Doing::Standing;
                        bot.until = now + stay_length(bot.at);
                    }
                    Then::Inn => {
                        bot.doing = Doing::Inside { at: to };
                        bot.until =
                            now + Duration::from_secs(fastrand::u64(INN_STAY.0..=INN_STAY.1));
                    }
                    Then::Exit { to: territory, pop } => {
                        bot.doing = Doing::Away {
                            to: territory,
                            arrive: Arrive::Pop(pop),
                        };
                        bot.until = now + Duration::from_secs(fastrand::u64(LOADING.0..=LOADING.1));
                    }
                }
            }
        }
        Doing::Teleporting { to, aetheryte } => {
            if !bot.cast_shown {
                bot.cast_shown = true;
                if let (Some(actor), Some(instance)) = (bot.actor, instance) {
                    // Up first if it was sitting (it joined a party from a seat.)
                    set_character_mode(
                        instance,
                        &mut network.lock(),
                        actor,
                        CharacterMode::Normal,
                        0,
                        true,
                    );
                    show_teleport_cast(actor, instance, network, bot.facing);
                }
            }
            if now >= bot.until {
                bot.cast_shown = false;
                bot.doing = Doing::Away {
                    to,
                    arrive: Arrive::Aetheryte(aetheryte),
                };
                bot.until = now + Duration::from_secs(fastrand::u64(LOADING.0..=LOADING.1));
            }
        }
        Doing::Away { to, arrive } => {
            if now >= bot.until && bot.joined {
                // Gone to a player's party: someone new comes home in its place.
                bot.joined = false;
                if let Some(game_data) = game_data {
                    let who = world_bots.make_who(&mut game_data.lock());
                    let bot = &mut world_bots.bots[index];
                    if let Some(who) = who {
                        bot.who = who;
                    }
                }
                let bot = &mut world_bots.bots[index];
                bot.doing = Doing::Away {
                    to: bot.home.0,
                    arrive,
                };
                bot.until = now + Duration::from_secs(fastrand::u64(LOADING.0..=LOADING.1));
                return;
            }
            if now >= bot.until {
                bot.at = to;
                let places = world_bots.places.get(&to).cloned();
                let bot = &mut world_bots.bots[index];
                bot.spot = places.as_deref().and_then(|places| match arrive {
                    Arrive::Pop(pop) => places.pops.get(&pop).copied(),
                    Arrive::Aetheryte(row) => places
                        .aetherytes
                        .iter()
                        .find(|(id, ..)| *id == row)
                        .and_then(|(_, at, landing)| landing.or(Some(*at))),
                });
                bot.zone_in_at =
                    matches!(arrive, Arrive::Aetheryte(_)).then_some(now + ZONE_IN_AFTER);
                bot.doing = Doing::Standing;
                // A moment where it arrived, then off to a spot of its own (`decide`.)
                bot.arrived = true;
                bot.until =
                    now + Duration::from_secs(fastrand::u64(ARRIVED_STAY.0..=ARRIVED_STAY.1));
            }
        }
        Doing::Inside { at } => {
            if now >= bot.until {
                bot.spot = Some(at);
                bot.doing = Doing::Standing;
                // Out of the inn, it soon goes on (home, or elsewhere.)
                bot.until = now + Duration::from_secs(fastrand::u64(3..=10));
            }
        }
    }
}

/// Its next thing, when a stay is over: another spot by its place (most often), or somewhere else (a place of its zone,
/// an inn, an exit, a Teleport), or home again if it's away.
fn decide(
    world_bots: &mut WorldBots,
    index: usize,
    places: Option<&Places>,
    mut instance: Option<&mut Instance>,
    network: &Arc<Mutex<NetworkState>>,
    settings: &Settings,
    now: Instant,
) {
    let territory = world_bots.bots[index].at;
    let teleport_to = teleport_destination(world_bots, territory, settings);
    let taken = taken_spots(&world_bots.bots, territory, index);
    let seats_taken = taken_seats(&world_bots.bots, territory, index);
    let bot = &mut world_bots.bots[index];
    // Up first, if it was sitting (and not straight down again.)
    let was_sitting = stand_up(bot, instance.as_deref_mut(), network);
    let ground = instance.as_deref().map(Ground::of).unwrap_or_default();
    let navmesh = ground.navmesh;
    let here = bot.spot;

    // Just arrived where everyone lands: off to a spot of its own by the zone's aetheryte (its own place, at home.)
    if std::mem::take(&mut bot.arrived)
        && let (Some(places), Some(here)) = (places, here)
    {
        let anchor = if bot.home.0 == bot.at {
            bot.home.1
        } else {
            Anchor::Aetheryte(places.aetherytes.first().map(|(id, ..)| *id).unwrap_or(0))
        };
        if let Some(to) = spot_by(bot.at, places, anchor, ground, &taken)
            && to.distance(here) > APART
        {
            start_walk(bot, here, to, Then::Stand, navmesh, now);
            return;
        }
    }
    let at_home = bot.home.0 == bot.at;
    let stay = |bot: &mut WorldBot| {
        bot.doing = Doing::Standing;
        bot.until = now + stay_length(bot.at);
    };

    // Going home.
    if !at_home {
        if settings.travel && fastrand::f32() < GOES_HOME {
            let (home, anchor) = bot.home;
            let aetheryte = match anchor {
                Anchor::Aetheryte(row) => Some(row),
                Anchor::Marketboard(_) => world_bots
                    .aetherytes
                    .iter()
                    .find(|place| place.big && place.territory == home)
                    .map(|place| place.id),
            };
            let bot = &mut world_bots.bots[index];
            if let Some(aetheryte) = aetheryte {
                bot.doing = Doing::Teleporting {
                    to: home,
                    aetheryte,
                };
                bot.until = now + TELEPORT_GONE;
                return;
            }
        }
        let bot = &mut world_bots.bots[index];
        stay(bot);
        return;
    }

    // Somewhere else.
    if settings.travel && fastrand::f32() < GOES_ELSEWHERE {
        if fastrand::f32() < LEAVES_BY_TELEPORT * 2.0
            && let Some((to, aetheryte)) = teleport_to
        {
            bot.doing = Doing::Teleporting { to, aetheryte };
            bot.until = now + TELEPORT_GONE;
            return;
        }
        if let (Some(places), Some(here)) = (places, here)
            && let Some((to, then)) = trip_in_zone(places, here)
            && let Some(to) = near_place(ground, to, then, places.landing(), &taken)
        {
            start_walk(bot, here, to, then, navmesh, now);
            return;
        }
    }

    // A seat near it, or the ground where it stands.
    if let Some(here) = here.filter(|_| !was_sitting) {
        if fastrand::f32() < GOES_TO_SIT
            && let Some((seat, place)) = free_seat(bot.at, here, &seats_taken)
        {
            // Up to it (where retail's players got up), then onto it.
            start_walk(bot, here, before_seat(place), Then::Sit(seat), navmesh, now);
            return;
        }
        if fastrand::f32() < SITS_DOWN {
            sit(bot, instance, network, ON_GROUND, None);
            stay(bot);
            return;
        }
    }

    // Another spot by its place.
    if let (Some(places), Some(here)) = (places, here)
        && let Some(to) = spot_by(bot.at, places, bot.home.1, ground, &taken)
        && to.distance(here) > 1.0
    {
        start_walk(bot, here, to, Then::Stand, navmesh, now);
        return;
    }
    stay(bot);
}

/// Sets off from `from` to `to`: on the navmesh when shown, else it just takes as long as the run would.
fn start_walk(
    bot: &mut WorldBot,
    from: Vec3A,
    to: Vec3A,
    then: Then,
    navmesh: Option<&crate::navmesh::Navmesh>,
    now: Instant,
) {
    bot.doing = Doing::Walking { to, then };
    bot.path = match (bot.actor, navmesh) {
        (Some(_), Some(navmesh)) => party_bot::instance_navmesh_path(navmesh, from, to).into(),
        _ => VecDeque::new(),
    };
    let length = from.distance(to) * 1.3;
    bot.until = now + Duration::from_secs_f32(length / NPC_RUN_SPEED + 1.0);
}

/// A tick of its walk while shown: a run along its path, sent to those who see it. Returns whether it got there.
fn walk(
    bot: &mut WorldBot,
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    to: Vec3A,
) -> bool {
    let Some(here) = instance.find_actor(actor).map(|actor| actor.position().0) else {
        return true;
    };
    if bot.path.is_empty() {
        if here.distance(to) < 1.0 {
            return true;
        }
        bot.path = party_bot::instance_navmesh_path(&instance.navmesh, here, to).into();
        if bot.path.is_empty() {
            bot.path.push_back(to);
        }
    }
    let (moved, _) = party_bot::run_along(
        &instance.navmesh,
        instance.collision.as_deref(),
        here,
        &mut bot.path,
        Duration::ZERO,
    );
    match moved {
        Some((position, facing)) => {
            bot.facing = facing;
            if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(actor) {
                spawn.common.position = Position(position);
                spawn.common.rotation = facing;
            }
            network.lock().send_in_range_instance(
                actor,
                instance,
                FromServer::ActorMove(
                    actor,
                    Position(position),
                    facing,
                    MoveAnimationType::empty(),
                    MoveAnimationState::None,
                    JumpState::NoneOrFalling,
                    Some((NPC_RUN_SPEED * 10.0) as u8),
                ),
                DestinationNetwork::ZoneClients,
            );
            bot.path.is_empty()
        }
        // Stuck (nothing gained this tick, path or not): it's as far as it gets.
        None => {
            bot.path.clear();
            true
        }
    }
}

/// Sits it down as retail's players sat (Bob's Chronofoil captures): in a chair, an ActorSetPos onto the seat facing its
/// way (warp type 19 with 1), the emote 50 naming that facing, then SetMode 11 with 2; on the ground, the emote 52 and
/// SetMode 11 with 1 where it stands. Unshown, it's just sitting when it's next shown.
fn sit(
    bot: &mut WorldBot,
    instance: Option<&mut Instance>,
    network: &Arc<Mutex<NetworkState>>,
    how: u8,
    seat: Option<usize>,
) {
    bot.sitting = Some(how);
    bot.seat = seat;
    let (Some(actor), Some(instance)) = (bot.actor, instance) else {
        return;
    };
    let in_chair = how == IN_CHAIR;
    if in_chair && let Some(spot) = bot.spot {
        set_pos(actor, instance, network, spot, bot.facing, SEAT_WARP_SIT);
    }
    emote_seated(
        actor,
        instance,
        network,
        if in_chair {
            SIT_EMOTE
        } else {
            GROUND_SIT_EMOTE
        },
        in_chair.then_some(bot.facing),
    );
    set_character_mode(
        instance,
        &mut network.lock(),
        actor,
        CharacterMode::InPositionLoop,
        how,
        true,
    );
}

/// Gets it up if it's sitting, as retail's players got up: from a chair, an ActorSetPos to just ahead of the seat
/// (warp type 19 with 2), SetMode 1, then the emote 51 naming the seat's facing; from the ground, SetMode 1 and the
/// emote 53. Whether it was sitting.
fn stand_up(
    bot: &mut WorldBot,
    instance: Option<&mut Instance>,
    network: &Arc<Mutex<NetworkState>>,
) -> bool {
    let Some(how) = bot.sitting.take() else {
        return false;
    };
    bot.seat = None;
    let in_chair = how == IN_CHAIR;
    if in_chair && let Some(spot) = bot.spot {
        let before = before_seat([spot.x, spot.y, spot.z, bot.facing]);
        bot.spot = Some(before);
        if let (Some(actor), Some(instance)) = (bot.actor, instance) {
            set_pos(
                actor,
                instance,
                network,
                before,
                bot.facing,
                SEAT_WARP_STAND,
            );
            set_character_mode(
                instance,
                &mut network.lock(),
                actor,
                CharacterMode::Normal,
                0,
                true,
            );
            emote_seated(actor, instance, network, STAND_EMOTE, Some(bot.facing));
        }
        return true;
    }
    if let (Some(actor), Some(instance)) = (bot.actor, instance) {
        set_character_mode(
            instance,
            &mut network.lock(),
            actor,
            CharacterMode::Normal,
            0,
            true,
        );
        emote_seated(actor, instance, network, GROUND_STAND_EMOTE, None);
    }
    true
}

/// ActorSetPos for a seat (warp type 19, `arg` 1 sitting, 2 getting up), and the actor's spawn there.
fn set_pos(
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    at: Vec3A,
    facing: f32,
    arg: u8,
) {
    if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(actor) {
        spawn.common.position = Position(at);
        spawn.common.rotation = facing;
    }
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
        rotation: facing,
        warp_type: WarpType::Unk19,
        arg,
        position: Position(at),
        ..Default::default()
    }));
    network.lock().send_in_range_instance(
        actor,
        instance,
        FromServer::PacketSegment(ipc, actor),
        DestinationNetwork::ZoneClients,
    );
}

/// A sitting or getting-up emote, as retail's: its text hidden, and for a chair the seat's facing quantized as
/// ActorSetPos' rotation (its fourth parameter.)
fn emote_seated(
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    emote: u32,
    facing: Option<f32>,
) {
    let seat_facing = facing
        .map(|facing| {
            (((facing + std::f32::consts::PI) / std::f32::consts::TAU) * u16::MAX as f32) as u16
                as u32
        })
        .unwrap_or(0);
    network.lock().send_in_range_instance(
        actor,
        instance,
        FromServer::ActorControlTarget(
            actor,
            ObjectTypeId::default(),
            ActorControlCategory::Emote {
                emote,
                hide_text: true,
                seat_facing,
            },
        ),
        DestinationNetwork::ZoneClients,
    );
}

/// While it stands: now and then a new idle pose or an emote, as often as retail's players; an emote meant for someone
/// at another bot near it. (Sitting, it just sits.)
fn idle(
    bot: &mut WorldBot,
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    emotes: &[(u32, u32, u8)],
) {
    if bot.sitting.is_some() {
        return;
    }
    let tick_hours = crate::server::npc_behavior::BEHAVIOR_TICK_SECONDS / 3600.0;
    if fastrand::f32() < POSE_CHANGES_PER_HOUR * tick_hours {
        bot.who.pose = (bot.who.pose + 1) % 7;
        if let Some(NetworkedActor::Npc {
            shown_as_player: Some(shown),
            ..
        }) = instance.find_actor_mut(actor)
        {
            shown.pose = bot.who.pose;
        }
        network.lock().send_in_range_instance(
            actor,
            instance,
            FromServer::ActorControl(
                actor,
                ActorControlCategory::Pose {
                    unk1: 0,
                    pose: bot.who.pose as u32,
                },
            ),
            DestinationNetwork::ZoneClients,
        );
    }
    if fastrand::f32() < EMOTES_PER_HOUR * tick_hours
        && let Some((emote, target)) = emote_to_use(emotes, actor, instance)
    {
        network.lock().send_in_range_instance(
            actor,
            instance,
            FromServer::ActorControlTarget(
                actor,
                target,
                ActorControlCategory::Emote {
                    emote,
                    hide_text: false,
                    seat_facing: 0,
                },
            ),
            DestinationNetwork::ZoneClients,
        );
    }
}

/// An emote for `actor` to use, by retail's shares, and whom at: one meant for someone goes to another world bot within
/// `EMOTE_AT_WITHIN`, or isn't used when there's none.
fn emote_to_use(
    emotes: &[(u32, u32, u8)],
    actor: ObjectId,
    instance: &Instance,
) -> Option<(u32, ObjectTypeId)> {
    let choices: Vec<((u32, u8), u16)> = emotes
        .iter()
        .map(|(id, n, aimed)| ((*id, *aimed), *n as u16))
        .collect();
    let (emote, aimed) = weighted(&choices)?;
    if fastrand::u8(..100) >= aimed {
        return Some((emote, ObjectTypeId::default()));
    }
    let here = instance.find_actor(actor)?.position().0;
    let near: Vec<ObjectId> = instance
        .world_bots
        .iter()
        .copied()
        .filter(|other| {
            *other != actor
                && instance
                    .find_actor(*other)
                    .is_some_and(|other| other.position().0.distance(here) <= EMOTE_AT_WITHIN)
        })
        .collect();
    (!near.is_empty()).then(|| {
        (
            emote,
            ObjectTypeId {
                object_id: near[fastrand::usize(..near.len())],
                object_type: ObjectTypeKind::None,
            },
        )
    })
}

/// Its Teleport cast, as others see a player's: Teleport (5), 5 s.
fn show_teleport_cast(
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    facing: f32,
) {
    let Some(position) = instance.find_actor(actor).map(|actor| actor.position()) else {
        return;
    };
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
        spell_id: TELEPORT_ACTION as u16,
        action_type: ActionType::Action,
        omen_delay: 0,
        action_id: TELEPORT_ACTION,
        cast_time: TELEPORT_CAST,
        target: actor,
        rotation: facing,
        interruptible: false,
        ballista_entity_id: ObjectId::default(),
        position,
    });
    network.lock().send_in_range_instance(
        actor,
        instance,
        FromServer::PacketSegment(ipc, actor),
        DestinationNetwork::ZoneClients,
    );
}

/// Puts it in its zone's instance at `spot`, shown as a player.
fn show(
    bot: &mut WorldBot,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    spot: Vec3A,
) {
    let _ = network;
    let actor = loop {
        let id = ObjectId(fastrand::u32(0x1000_0000..0x2000_0000));
        if instance.find_actor(id).is_none() {
            break id;
        }
    };
    let mut spawn = bot.who.spawn.clone();
    spawn.common.position = Position(spot);
    spawn.common.rotation = bot.facing;
    if let Some(how) = bot.sitting {
        spawn.common.mode = CharacterMode::InPositionLoop;
        spawn.common.mode_arg = how;
    }
    let config = get_config();
    instance.insert_npc(actor, spawn, &config, &mut game_data.lock());
    let content_id = bot.who.content_id;
    if let Some(NetworkedActor::Npc {
        timeline,
        state,
        shown_as_player,
        ..
    }) = instance.find_actor_mut(actor)
    {
        timeline.hitbox_radius = 0.5;
        *state = NpcState::Wander;
        *shown_as_player = Some(ShownAsPlayer {
            account_id: content_id,
            content_id,
            world_id: config.world.world_id,
            handler_id: Default::default(),
            online_status: Some(bot.who.online_status),
            title_id: bot.who.title_id,
            pose: bot.who.pose,
        });
    } else {
        return;
    }
    instance.world_bots.insert(actor);
    bot.actor = Some(actor);
    bot.spot = Some(spot);
    // Mid-walk, it walks on from here.
    bot.path.clear();
}

/// Who the shown world bot `actor` is (Examine.)
pub fn shown(bots: &WorldBots, actor: ObjectId) -> Option<&Who> {
    bots.bots
        .iter()
        .find(|bot| bot.actor == Some(actor))
        .map(|bot| &bot.who)
}

/// The world bot a player's menu named (its content ID, wherever it is; with none, its name among those shown in
/// `territory`), as a party bot would be kept (`bot_friends::SavedBot`): befriended, or called into a party. A crafter or gatherer comes as one of
/// the fighting jobs, drawn as the others' are.
pub fn as_friend(
    bots: &WorldBots,
    territory: Option<u16>,
    content_id: u64,
    name: &str,
) -> Option<(usize, SavedBot)> {
    let (index, bot) = bots.bots.iter().enumerate().find(|(_, bot)| {
        if bot.joined {
            false
        } else if content_id != 0 {
            bot.who.content_id == content_id
        } else {
            bot.actor.is_some()
                && !name.is_empty()
                && bot.who.name == name
                && Some(bot.at) == territory
        }
    })?;
    let fighting: Vec<(u8, u16)> = JOBS
        .iter()
        .copied()
        .filter(|(job, _)| !DOH_DOL.contains(job))
        .collect();
    let classjob = if DOH_DOL.contains(&bot.who.classjob) {
        weighted(&fighting).unwrap_or(1)
    } else {
        bot.who.classjob
    };
    Some((
        index,
        SavedBot {
            content_id: bot.who.content_id,
            name: bot.who.name.clone(),
            classjob,
            look: party_bot::look_to_hex(&bot.who.look),
            glamour: bot.who.glamour.clone(),
            added: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|time| time.as_secs() as u32)
                .unwrap_or_default(),
        },
    ))
}

/// The world bot at `index` joined a player's party (a party bot beside them now): it Teleports away, and someone new
/// takes its place at home.
pub fn joins_party(bots: &mut WorldBots, index: usize, now: Instant) {
    let aetheryte = bots
        .aetherytes
        .iter()
        .find(|place| place.big && place.territory == bots.bots[index].home.0)
        .map(|place| place.id)
        .unwrap_or(0);
    let bot = &mut bots.bots[index];
    let aetheryte = match bot.home.1 {
        Anchor::Aetheryte(row) => row,
        Anchor::Marketboard(_) => aetheryte,
    };
    bot.sitting = None;
    bot.seat = None;
    bot.joined = true;
    bot.cast_shown = false;
    bot.doing = Doing::Teleporting {
        to: bot.home.0,
        aetheryte,
    };
    bot.until = now + TELEPORT_GONE;
}

/// The world bot at `index`, shown in the inviting player's own zone, became their party bot where it stood (its
/// actor gone, a party bot in its place: `party_bot`'s invite): it's no longer one of the crowd, and someone new comes
/// home in its place after a while.
pub fn leaves_for_party(bots: &mut WorldBots, index: usize, now: Instant) {
    let aetheryte = bots
        .aetherytes
        .iter()
        .find(|place| place.big && place.territory == bots.bots[index].home.0)
        .map(|place| place.id)
        .unwrap_or(0);
    let bot = &mut bots.bots[index];
    let aetheryte = match bot.home.1 {
        Anchor::Aetheryte(row) => row,
        Anchor::Marketboard(_) => aetheryte,
    };
    bot.actor = None;
    bot.path.clear();
    bot.sitting = None;
    bot.seat = None;
    bot.joined = true;
    bot.doing = Doing::Away {
        to: bot.home.0,
        arrive: Arrive::Aetheryte(aetheryte),
    };
    bot.until = now + Duration::from_secs(INN_STAY.0);
}

/// What `!worldbots` prints: whether they're on, how many and where, and the nearest to `player` with what each is
/// doing.
pub fn report(data: &WorldServer, player: ObjectId) -> Vec<String> {
    let bots = &data.world_bots;
    let settings = bots.settings();
    let mut lines = vec![format!(
        "World bots {} ({}): {} in all, {} shown",
        if settings.enabled { "on" } else { "off" },
        settings_path().display(),
        bots.bots.len(),
        bots.bots.iter().filter(|bot| bot.actor.is_some()).count()
    )];
    let mut by_zone: HashMap<u16, usize> = HashMap::new();
    for bot in &bots.bots {
        *by_zone.entry(bot.at).or_default() += 1;
    }
    let mut by_zone: Vec<(u16, usize)> = by_zone.into_iter().collect();
    by_zone.sort_by_key(|(territory, count)| (std::cmp::Reverse(*count), *territory));
    lines.push(format!(
        "Most: {}",
        by_zone
            .iter()
            .take(8)
            .map(|(territory, count)| format!("{territory}: {count}"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    let Some(instance) = data.find_actor_instance(player) else {
        return lines;
    };
    let Some(here) = instance.find_actor(player).map(|actor| actor.position().0) else {
        return lines;
    };
    if let Some(places) = bots.places.get(&instance.zone.id) {
        lines.push(format!(
            "Here: {} big aetherytes, {} shards, {} marketboards, {} innkeepers, {} exits",
            places.aetherytes.len(),
            places.shards.len(),
            places.marketboards.len(),
            places.inns.len(),
            places.exits.len()
        ));
    }
    let mut near: Vec<(f32, &WorldBot)> = bots
        .bots
        .iter()
        .filter(|bot| bot.at == instance.zone.id)
        .filter_map(|bot| Some((bot.spot?.distance(here), bot)))
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (distance, bot) in near.iter().take(5) {
        let doing = match &bot.doing {
            Doing::Standing => "standing".to_string(),
            Doing::Walking { then, .. } => format!("walking ({then:?})"),
            Doing::Teleporting { to, .. } => format!("teleporting to {to}"),
            Doing::Away { to, .. } => format!("on the way to {to}"),
            Doing::Inside { .. } => "in an inn room".to_string(),
        };
        lines.push(format!(
            "{} (class {}, level {}, home {}): {doing}, {distance:.0} yalms away",
            bot.who.name, bot.who.classjob, bot.who.level, bot.home.0
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_from_the_file_and_their_defaults() {
        let settings = Settings::from_json(&serde_json::json!({
            "enabled": true,
            "cities": {"129": 30, "130": 10, "bad": 3},
            "per_marketboard": [10, 5],
            "travel": false
        }));
        assert!(settings.enabled);
        assert_eq!(settings.cities, vec![(129, 30), (130, 10)]);
        assert_eq!(settings.per_aetheryte, 1);
        assert_eq!(settings.per_marketboard, (5, 10));
        assert!(!settings.travel);
        assert_eq!(Settings::from_json(&Value::Null), Settings::default());
        assert!(!Settings::default().enabled);
    }

    #[test]
    fn stays_are_as_long_as_retails() {
        let mut lengths: Vec<f32> = (0..4000).map(|_| stay_length(129).as_secs_f32()).collect();
        lengths.sort_by(f32::total_cmp);
        let median = lengths[lengths.len() / 2];
        assert!((45.0..65.0).contains(&median), "{median}");
        assert!(lengths[0] >= 8.0);
    }

    #[test]
    fn retail_spots_are_read() {
        let spots = recorded_spots();
        assert!(spots.get(&129).is_some_and(|cells| cells.len() > 20));
        // Limsa Lominsa's aetheryte plaza is the busiest.
        assert!(spots[&129][0][3] > 10000.0);
    }

    /// Nobody watching: a bot that teleports away is gone for its cast and a loading screen, arrives in the other zone
    /// and stands there, and in time comes home again (by Teleport); it never gets stuck.
    #[test]
    fn a_bot_travels_and_comes_home_unwatched() {
        fastrand::seed(7);
        let now = Instant::now();
        let mut bots = WorldBots {
            settings: Some((
                None,
                Settings {
                    enabled: true,
                    ..Settings::default()
                },
            )),
            aetherytes: vec![
                AetheryteRowPlace {
                    id: 8,
                    territory: 129,
                    big: true,
                    levels: vec![],
                },
                AetheryteRowPlace {
                    id: 2,
                    territory: 132,
                    big: true,
                    levels: vec![],
                },
            ],
            ready: true,
            ..Default::default()
        };
        let who = Who {
            name: "Test Bot".into(),
            classjob: 1,
            level: 10,
            look: CustomizeData::default(),
            spawn: SpawnNpc::default(),
            online_status: OnlineStatus::Offline,
            title_id: 0,
            pose: 0,
            equipped: EquippedStorage::default(),
            item_level: 0,
            glamour: None,
            content_id: CONTENT_ID_BASE | 1,
        };
        bots.bots.push(WorldBot {
            who,
            home: (129, Anchor::Aetheryte(8)),
            at: 129,
            spot: None,
            facing: 0.0,
            doing: Doing::Teleporting {
                to: 132,
                aetheryte: 2,
            },
            until: now,
            actor: None,
            path: VecDeque::new(),
            zone_in_at: None,
            cast_shown: false,
            sitting: None,
            seat: None,
            joined: false,
            arrived: false,
        });
        let network = Arc::new(Mutex::new(NetworkState::default()));
        let settings = bots.settings();
        let mut clock = now;
        let mut away = false;
        let mut came_home = false;
        for _ in 0..20000 {
            clock += Duration::from_millis(500);
            step(&mut bots, 0, None, &network, None, &settings, clock);
            let bot = &bots.bots[0];
            if bot.at == 132 && bot.doing == Doing::Standing {
                away = true;
            }
            if away && bot.at == 129 {
                came_home = true;
                break;
            }
        }
        assert!(away, "it arrived in the other zone");
        assert!(came_home, "and came home");
    }

    /// Monsters never go after a world bot.
    #[test]
    fn enemies_leave_world_bots_alone() {
        let mut world = crate::server::test_world::TestWorld::new();
        let player = world.player(1, [0.0, 0.0, 0.0]);
        let bot = world.player_npc(2, player, [2.0, 0.0, 0.0]);
        world.instance.party_bots.remove(&bot);
        world.instance.world_bots.insert(bot);
        let possible: Vec<ObjectId> = world
            .instance
            .find_possible_enemies()
            .into_iter()
            .map(|(id, ..)| id)
            .collect();
        assert!(possible.contains(&player));
        assert!(!possible.contains(&bot));
    }

    /// Bob's bundle 20261111-082127: a bot stood under the floor (the navmesh's lower layer) where retail's players
    /// stood at 1.28. On the collision's floor near the recorded height instead; kept as recorded when there's none.
    #[test]
    fn a_spot_stands_on_the_floor_near_its_height() {
        let (mut v, mut t) = (Vec::new(), Vec::new());
        for y in [-0.13, 1.3] {
            let i = v.len() as u32;
            v.extend([
                [10.0, y, 20.0],
                [30.0, y, 20.0],
                [30.0, y, 30.0],
                [10.0, y, 30.0],
            ]);
            t.extend([[i, i + 1, i + 2], [i, i + 2, i + 3]]);
        }
        let collision =
            crate::collision::Collision::build(v, t, Vec::new(), Vec::new(), 0.75, 0.4, 55.0);
        let ground = Ground {
            navmesh: None,
            collision: Some(&collision),
        };
        let spot = ground.recorded(Vec3A::new(19.04, 1.28, 26.06));
        assert!((spot.y - 1.3).abs() < 1e-3, "{spot:?}");
        // Nothing near that height: as recorded.
        let spot = ground.recorded(Vec3A::new(19.04, 5.0, 26.06));
        assert_eq!(spot.y, 5.0);
    }

    /// Bob's bundle 20261111-091633: Thalmiel Thalis spawned at 0.30 on New Gridania's bridge, where retail's players
    /// stood at 1.28: the zone's collision has no bridge there, only the floor under it. A floor more than a quarter
    /// yalm from the recorded height isn't taken.
    #[test]
    fn a_recorded_spot_keeps_its_height_over_a_floor_below() {
        let collision = crate::collision::Collision::build(
            vec![
                [10.0, 0.30, 20.0],
                [30.0, 0.30, 20.0],
                [30.0, 0.30, 30.0],
                [10.0, 0.30, 30.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
            Vec::new(),
            Vec::new(),
            0.75,
            0.4,
            55.0,
        );
        let ground = Ground {
            navmesh: None,
            collision: Some(&collision),
        };
        let spot = ground.recorded(Vec3A::new(18.36, 1.28, 25.76));
        assert_eq!(spot.y, 1.28);
    }

    /// Bob's bundle 20261111-091633: bots that arrived stood where everyone lands, in a heap. A moment there, then off
    /// to a spot of their own.
    #[test]
    fn an_arrival_walks_off_from_the_landing() {
        fastrand::seed(5);
        let landing = Vec3A::new(-80.0, 20.0, 0.0);
        let mut bots = WorldBots {
            settings: Some((None, Settings::default())),
            ready: true,
            ..Default::default()
        };
        bots.places.insert(
            999,
            Arc::new(Places {
                aetherytes: vec![(8, Vec3A::new(-84.0, 20.0, 0.0), Some(landing))],
                ..Default::default()
            }),
        );
        let mut bot = test_bot(999);
        bot.home = (999, Anchor::Aetheryte(8));
        let now = Instant::now();
        bot.doing = Doing::Away {
            to: 999,
            arrive: Arrive::Aetheryte(8),
        };
        bot.until = now;
        bots.bots.push(bot);
        let network = Arc::new(Mutex::new(NetworkState::default()));
        let settings = bots.settings();
        step(&mut bots, 0, None, &network, None, &settings, now);
        assert_eq!(bots.bots[0].spot, Some(landing));
        assert!(bots.bots[0].arrived);
        let later = now + Duration::from_secs(ARRIVED_STAY.1 + 1);
        step(&mut bots, 0, None, &network, None, &settings, later);
        let Doing::Walking { to, then } = bots.bots[0].doing.clone() else {
            panic!("still at the landing: {:?}", bots.bots[0].doing);
        };
        assert_eq!(then, Then::Stand);
        assert!(to.distance(landing) > APART);
    }

    /// Bob's bundle 20261111-082127: bots stood in one another.
    #[test]
    fn bots_stand_apart() {
        fastrand::seed(3);
        let places = Places {
            marketboards: vec![Vec3A::new(10.0, 0.0, 10.0)],
            ..Default::default()
        };
        let mut taken = Vec::new();
        for _ in 0..10 {
            let spot = spot_by(
                999,
                &places,
                Anchor::Marketboard(0),
                Ground::default(),
                &taken,
            )
            .unwrap();
            assert!(
                taken
                    .iter()
                    .all(|other: &Vec3A| other.distance(spot) >= APART),
                "{spot:?} in {taken:?}"
            );
            taken.push(spot);
        }
    }

    /// Retail's seats are read, and a taken one isn't offered.
    #[test]
    fn a_free_seat() {
        let gridania = seats().get(&132).expect("New Gridania's seats");
        assert!(!gridania.is_empty());
        let [x, y, z, _] = gridania[0];
        let here = Vec3A::new(x, y, z);
        let all: HashSet<usize> = (0..gridania.len()).collect();
        assert!(free_seat(132, here, &all).is_none());
        let mut taken = all.clone();
        taken.remove(&0);
        assert_eq!(
            free_seat(132, here, &taken).map(|(index, _)| index),
            Some(0)
        );
    }

    fn test_bot(at: u16) -> WorldBot {
        WorldBot {
            who: Who {
                name: "Test Bot".into(),
                classjob: 1,
                level: 10,
                look: CustomizeData::default(),
                spawn: SpawnNpc::default(),
                online_status: OnlineStatus::Offline,
                title_id: 0,
                pose: 0,
                equipped: EquippedStorage::default(),
                item_level: 0,
                glamour: None,
                content_id: CONTENT_ID_BASE | 2,
            },
            home: (at, Anchor::Aetheryte(2)),
            at,
            spot: None,
            facing: 0.0,
            doing: Doing::Standing,
            until: Instant::now(),
            actor: None,
            path: VecDeque::new(),
            zone_in_at: None,
            cast_shown: false,
            sitting: None,
            seat: None,
            joined: false,
            arrived: false,
        }
    }

    /// Bob's bundle 20261111-082127: bots should sit on benches. Sitting as retail's players: the emote Sit, then
    /// SetMode 11 with 2; up again (SetMode Normal) when its stay is over.
    #[test]
    fn a_bot_sits_in_a_seat_and_gets_up() {
        use crate::server::test_world::{TestWorld, control_from, packet_from};
        let mut world = TestWorld::new();
        let player = world.player(1, [0.0, 0.0, 0.0]);
        let actor = world.player_npc(2, player, [1.0, 0.0, 0.0]);
        world.instance.party_bots.remove(&actor);
        world.instance.world_bots.insert(actor);
        let [x, y, z, facing] = seats()[&132][0];
        let mut bots = WorldBots {
            settings: Some((None, Settings::default())),
            ready: true,
            ..Default::default()
        };
        let mut bot = test_bot(132);
        bot.actor = Some(actor);
        bot.spot = Some(Vec3A::new(1.0, 0.0, 0.0));
        bot.doing = Doing::Walking {
            to: Vec3A::new(x, y, z),
            then: Then::Sit(0),
        };
        bots.bots.push(bot);
        let network = world.network.clone();
        let settings = bots.settings();
        let now = Instant::now();
        world.sent(player);
        // Arrived (it was put on the seat: the test's instance has no navmesh to walk.)
        world
            .instance
            .find_actor_mut(actor)
            .unwrap()
            .get_common_spawn_mut()
            .position = Position(Vec3A::new(x, y, z));
        step(
            &mut bots,
            0,
            Some(&mut world.instance),
            &network,
            None,
            &settings,
            now,
        );
        let bot = &bots.bots[0];
        assert_eq!((bot.sitting, bot.seat), (Some(IN_CHAIR), Some(0)));
        assert_eq!(bot.facing, facing);
        let sent = world.sent(player);
        // Onto the seat facing its way, as retail's players (Bob's bundle 20261111-091633: Selis Kalomiyn sat a
        // third of a yalm off, facing the way it walked up.)
        assert!(
            sent.iter()
                .any(|message| packet_from(message, actor, |data| matches!(data,
            ServerZoneIpcData::ActorSetPos(set)
                if matches!(set.warp_type, WarpType::Unk19) && set.arg == SEAT_WARP_SIT
                    && set.position.0 == Vec3A::new(x, y, z) && set.rotation == facing)))
        );
        assert!(sent.iter().any(|message| matches!(message,
            FromServer::ActorControlTarget(from, _, ActorControlCategory::Emote { emote: SIT_EMOTE, seat_facing, .. })
                if *from == actor && *seat_facing != 0)));
        assert!(sent.iter().any(|message| matches!(message,
            FromServer::ActorControlTarget(from, _, ActorControlCategory::Emote { emote: SIT_EMOTE, .. })
                if *from == actor)));
        assert!(
            sent.iter()
                .any(|message| control_from(message, actor, |category| matches!(
                    category,
                    ActorControlCategory::SetMode {
                        mode: CharacterMode::InPositionLoop,
                        mode_arg: 2
                    }
                )))
        );
        // Its seat isn't offered to another.
        assert!(taken_seats(&bots.bots, 132, 1).contains(&0));
        // The stay's over: up.
        decide(
            &mut bots,
            0,
            None,
            Some(&mut world.instance),
            &network,
            &settings,
            now,
        );
        assert_eq!(bots.bots[0].sitting, None);
        let sent = world.sent(player);
        assert!(
            sent.iter()
                .any(|message| control_from(message, actor, |category| matches!(
                    category,
                    ActorControlCategory::SetMode {
                        mode: CharacterMode::Normal,
                        ..
                    }
                )))
        );
        // Put just ahead of the seat, and the emote of getting up.
        let ahead = before_seat([x, y, z, facing]);
        assert!(
            sent.iter()
                .any(|message| packet_from(message, actor, |data| matches!(data,
            ServerZoneIpcData::ActorSetPos(set)
                if set.arg == SEAT_WARP_STAND && set.position.0 == ahead)))
        );
        assert!(sent.iter().any(|message| matches!(message,
            FromServer::ActorControlTarget(from, _, ActorControlCategory::Emote { emote: STAND_EMOTE, .. })
                if *from == actor)));
    }

    /// Bob's bundle 20261111-082127: an emote such as a hug goes to another bot, or isn't used.
    #[test]
    fn a_hug_is_for_another_bot() {
        use crate::server::test_world::TestWorld;
        let mut world = TestWorld::new();
        let player = world.player(1, [0.0, 0.0, 0.0]);
        let actor = world.player_npc(2, player, [1.0, 0.0, 0.0]);
        world.instance.party_bots.remove(&actor);
        world.instance.world_bots.insert(actor);
        let hug = [(112, 1, 100)];
        assert_eq!(emote_to_use(&hug, actor, &world.instance), None);
        let other = world.player_npc(3, player, [3.0, 0.0, 0.0]);
        world.instance.party_bots.remove(&other);
        world.instance.world_bots.insert(other);
        let (emote, target) = emote_to_use(&hug, actor, &world.instance).unwrap();
        assert_eq!((emote, target.object_id), (112, other));
        // One not meant for anyone is at nobody.
        let (_, target) = emote_to_use(&[(5, 1, 0)], actor, &world.instance).unwrap();
        assert_eq!(target, ObjectTypeId::default());
    }

    /// Bob's bundles 20261111-084241 and -084610: "examine not working in overworld player bots", "sending friend
    /// requests to bots don't work either anymore". A world bot is examined as a party bot is (what it wears, its
    /// item level and title), and found from the menu's content ID or name to be kept as a friend or called into a
    /// party; joined, it Teleports off and someone new comes home in its place.
    #[test]
    fn a_world_bot_is_examined_befriended_and_joins_a_party() {
        use crate::server::{WorldServer, party_bot::examine_information, test_world::TestWorld};
        let mut world = TestWorld::new();
        let player = world.player(1, [0.0, 0.0, 0.0]);
        let actor = world.player_npc(2, player, [1.0, 0.0, 0.0]);
        world.instance.party_bots.remove(&actor);
        world.instance.world_bots.insert(actor);
        if let Some(NetworkedActor::Npc {
            shown_as_player: Some(shown),
            ..
        }) = world.instance.find_actor_mut(actor)
        {
            shown.title_id = 425;
        }
        let name = "Nyth Jael".to_string();
        world
            .instance
            .find_actor_mut(actor)
            .unwrap()
            .get_common_spawn_mut()
            .name = name.clone();
        let mut data = WorldServer::default();
        data.instances.push(std::mem::take(&mut world.instance));
        let mut bot = test_bot(129);
        bot.actor = Some(actor);
        bot.who.name = name.clone();
        bot.who.classjob = 8;
        bot.who.item_level = 123;
        data.world_bots.bots.push(bot);

        let Some(ServerZoneIpcData::ExamineCharacterInformation {
            unk1, name: shown, ..
        }) = examine_information(&data, player, actor)
        else {
            panic!("no Examine for a world bot");
        };
        assert_eq!(shown, name);
        assert_eq!(u16::from_le_bytes([unk1[6], unk1[7]]), 425);
        assert_eq!(u16::from_le_bytes([unk1[72], unk1[73]]), 123);

        // By its content ID, or by name in its zone (a menu's invite carries only the name.)
        let content_id = data.world_bots.bots[0].who.content_id;
        let (index, saved) = as_friend(&data.world_bots, Some(129), content_id, "").unwrap();
        assert_eq!((index, saved.content_id), (0, content_id));
        assert!(crate::server::bot_friends::is_bot_content_id(
            saved.content_id
        ));
        assert!(
            !DOH_DOL.contains(&saved.classjob),
            "a crafter comes as a fighter"
        );
        assert!(as_friend(&data.world_bots, Some(129), 0, &name).is_some());
        assert!(as_friend(&data.world_bots, Some(132), 0, &name).is_none());

        let now = Instant::now();
        // Called from the friend list while it's in the crowd here, it's the one that joins: it leaves the crowd.
        leaves_for_party(&mut data.world_bots, 0, now);
        assert!(data.world_bots.bots[0].actor.is_none() && data.world_bots.bots[0].joined);
        assert!(as_friend(&data.world_bots, Some(129), content_id, "").is_none());
        // Elsewhere, it Teleports off; someone new comes home in its place either way.
        data.world_bots.bots[0].joined = false;
        data.world_bots.bots[0].doing = Doing::Standing;
        joins_party(&mut data.world_bots, 0, now);
        let settings = data.world_bots.settings();
        let network = Arc::new(Mutex::new(NetworkState::default()));
        let mut clock = now;
        for _ in 0..100 {
            clock += Duration::from_millis(500);
            step(
                &mut data.world_bots,
                0,
                None,
                &network,
                None,
                &settings,
                clock,
            );
        }
        let bot = &data.world_bots.bots[0];
        assert!(!bot.joined);
        assert_eq!(bot.at, 129);
        assert_eq!(bot.doing, Doing::Standing);
    }

    #[test]
    fn a_spot_by_its_place() {
        let places = Places {
            aetherytes: vec![(8, Vec3A::new(-84.0, 20.0, 0.0), None)],
            marketboards: vec![Vec3A::new(10.0, 0.0, 10.0)],
            ..Default::default()
        };
        // Limsa Lominsa's recorded spots near its crystal.
        let spot = spot_by(129, &places, Anchor::Aetheryte(8), Ground::default(), &[]).unwrap();
        assert!(spot.distance(Vec3A::new(-84.0, 20.0, 0.0)) <= AETHERYTE_SPOTS_WITHIN + 1.0);
        // No recorded spots there: in the ring round the board.
        let spot = spot_by(999, &places, Anchor::Marketboard(0), Ground::default(), &[]).unwrap();
        let distance = spot.distance(Vec3A::new(10.0, 0.0, 10.0));
        assert!((MARKETBOARD_RING.0 - 0.01..=MARKETBOARD_RING.1 + 0.01).contains(&distance));
    }
}
