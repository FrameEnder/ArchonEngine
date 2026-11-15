//! Keybound Brawler (ContentFinderCondition 1095, InstanceContent 57001, territory 1359): the typing game of the
//! FFVII Gold Saucer event (festival 257, `game_events.rs`.)
//!
//! Enemies march on a barrier; each carries a word, and a word typed is an attack on the enemy carrying it. The client
//! does the typing: it tells the server only which enemy was hit (an ActionRequest for action 49516 or 49517 on it;
//! 49520 on the player is a word nobody carried.) The server runs the rest, as retail's did in **223 runs** (Connor's
//! Chronofoil captures of 2026-11-03 to 11-09) and four of Bob's (his Kawari Recorder export of 2026-11-12); the
//! numbers are in `resources/data/keybound_waves.json`, `docs/src/fork/events.md` and the working notes
//! (`notes/keybound-brawler.md` in kawari-diagnostics.)
//!
//! **The run.** Once the duty has begun (the opening over for everyone): the first wave's enemies appear 7.5 s later
//! (`APPEAR`, the director's 0x10000006, with phase 1), and the wave is on 5.0 s after that (`WAVE_ON`, 0x10000001,
//! phase 2.) A wave over: phase 0, `WAVE_CLEARED [wave]` (not the last), each player's bonus (10,000 for a whole
//! barrier, in proportion), `WAVE_OVER`, what's left of its enemies gone; 3.0 s later `BETWEEN`, LogMessage 11564 and
//! the barrier whole again (`BARRIER_SET [40000]`); 5.0 s after that `BETWEEN` again, and the next wave's enemies
//! appear 12.6 s after the wave ended (30.5 s before the last wave, where retail played a cutscene meanwhile.)
//!
//! **A wave** is a line-up picked at random (`keybound_waves.json`): each enemy has the moment its SpawnNpc comes and
//! the moment it gets its words and sets off (`WORDS [enemy, group, 2000010 + group, word, next word, 100]` and
//! ActorControl 54 `[1]`; 2.0 s before, ActorControl 407 `[7750]`, for those that wait.) It walks straight at the
//! barrier, stops 6 yalms short of it (z 11.5) and hits it at once and every 3.0 s after (the barrier's HP in
//! `BARRIER_NOW`.) The first wave ends at 25 kills; the second and third at their big enemy's fall.
//!
//! **A hit** (the client's ActionRequest): the ActionEffect for 100 damage, the player's score (`SCORE`; 100 a word,
//! 200 for the second wave's big enemy's, 300 for the last boss's, times 1.0 to 2.0 by the player's bonus level), the
//! bonus gauge (15 times the word's worth more; from 0.535 s later it drains) and the enemy's next words. The gauge
//! is the client's to draw from its own packet (`KeyboundGauge`: whether it's held, its value, the server's time),
//! sent with each word and as it starts to drain. An enemy at 0: ActorControls 14 and 2 `[2]`, `DEFEATED` and
//! `WORDS_GONE` for the one whose word did it, `ENEMY_DOWN [enemy, 3068]`, `KILLS`; 2.0 s later its words go for
//! everyone (`WORDS_GONE`, ActorControl 54 `[0]`), at 3.04 s ActorControl 607 `[enemy, 1, 0, 100]`, at 4.32 s it's
//! deleted.
//!
//! **Impassioned Inputs**: a 19592's fall gives everyone 20 s of words worth 1,000 with the gauge held
//! (`begin_fever`.) **The casters, the big enemies and what they summon** are in `keybound/play.rs`, from
//! `resources/data/keybound_mechanics.json`: casts (and the four words that break one with the counter), each big
//! enemy's rotation, its summons, the barrier's words and the shield they raise, the special states, and the lock a
//! client asks for its player.
//!
//! **The end**: the barrier at 0 loses it; the last boss's fall wins it (`finish`: the result's packet and scenes.)
//! Either way each player gets MGP and MGC by their score (`rewards`) and, for a win, a Keybound Coffer.
//!
//! What isn't known is in `docs/src/fork/known-issues.md`.

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use kawari::{
    common::{DirectorEvent, HandlerId, ObjectId, ObjectTypeId, ObjectTypeKind, Position},
    config::get_config,
    ipc::zone::{
        ActionEffect, ActionRequest, ActionType, ActorControlCategory, DamageElement, DamageKind,
        DamageType, EventLogMessage, ServerZoneIpcData, ServerZoneIpcSegment, TargetEffect,
        TargetEffectKind,
    },
};
use parking_lot::Mutex;
use serde_json::Value;

use crate::{
    ClientId, FromServer, GameData,
    server::{
        actor::{NetworkedActor, create_npc_common_spawn, kill_actor},
        director::LuaDirectorTask,
        effect::{gain_effect_instance, lose_effect_instance},
        instance::Instance,
        keybound_mechanics::{
            Add, MAP_ABSORB, MAP_BLOW, MAP_CHARGE, MAP_FALLEN, MAP_LOOK, MAP_SHIELD, Place, Spell,
            Then, What, barrier_look, mechanics, stand,
        },
        network::{DestinationNetwork, NetworkState},
    },
};

mod play;

/// The duty (its ContentFinderCondition.)
pub const DUTY: u16 = 1095;

/// A word that hit (the client alternates the two; under Impassioned Inputs it sends 51027 and 51028 instead) and
/// one nobody carried.
pub const ACTION_HITS: [u32; 4] = [49516, 49517, 51027, 51028];
pub const ACTION_MISS: u32 = 49520;
/// A word on the barrier's words (the client alternates these two as well): it fills them instead of hurting them.
pub const ACTION_CHARGES: [u32; 2] = [49518, 49519];

/// Whether an action is one of the game's words.
pub fn is_word(action: u32) -> bool {
    ACTION_HITS.contains(&action) || ACTION_CHARGES.contains(&action) || action == ACTION_MISS
}

/// The director's commands (ActorControlSelf 109 `[director, command, …]`), as retail sent them.
const WAVE_ON: u32 = 0x1000_0001;
const WAVE_OVER: u32 = 0x1000_0002;
const BARRIER_SET: u32 = 0x1000_0003;
const BARRIER_NOW: u32 = 0x1000_0004;
const APPEAR: u32 = 0x1000_0006;
const PLAYER: u32 = 0x1000_0007;
const WORDS: u32 = 0x1000_0008;
const BEGIN: u32 = 0x1000_0009;
/// A player in the game: their content ID, its high half first. One for each player as someone comes in, their own
/// first (Connor's runs: one to four of them right after ZoneInit, 375 different IDs, each the content ID the result's
/// packet has for a player.)
const PLAYERS: u32 = 0x1000_0015;
/// The client's director trigger as it loads, `[0x10000001, 1, 0, a number of its own]`: answered with the set-up's
/// packet and `BEGIN` (223 of 223: PLAYERS, the trigger, the set-up, BEGIN `[0]`, and only then the client's trigger
/// 801 for the director's variables, its EnterTerritoryEvent and the opening scene). Unanswered, the client waited
/// 45 s, never asked for the variables and never played the opening (Bob, 2026-11-15: "infinite loading".)
const TRIGGER_LOADED: (u32, u32) = (0x1000_0001, 1);
/// The set-up's numbers that never changed.
const SETUP: [u32; 3] = [6, 12_743_016, 12_743_021];
/// The game's places.
const PLACES: u32 = 4;
/// The status every player has from the first StatusEffectList of the duty to its end, from no one (80 of 80 runs
/// looked at: 5373 with 505), and the two conditions retail's Condition had for as long (`KEYBOUND_CONDITIONS` in
/// `zone_connection/zone.rs`.) Without them the client took no typing: "I typed something slowly and correctly and
/// got a can't use yet warning" (Bob, 2026-11-15.)
pub const PLAYER_STATUS: (u16, u16) = (5373, 505);
/// The director's sequence (DirectorVars' first byte) and the map effects' states as the duty loads
/// (DirectorSetupMapEffects64 `05 00 | 01 00 | 01 00`): retail's, 223 of 223. Kawari's were 0 and 4, 4 (4 is the
/// barrier fallen.)
const SEQUENCE: u8 = 1;
pub const OPENING_MAP_EFFECTS: [u16; 2] = [1, 1];
/// The banners (ActorControlSelf 123, ShowScreenImage `[image, 0xE0000000]`; 1,340 in 223 runs, in this order): each
/// wave's as it's announced (the first 2.0 s after the duty commences, the second 8.05 s after the first wave's end,
/// the last as the cutscene ends, 26.0 s after the second's), a wave's end with its `WAVE_OVER`, the one before the
/// cutscene (8.05 s after the second wave's end) and a loss's (3.0 s after its `WAVE_OVER`.)
const IMAGE_WAVE: [u32; 3] = [1620, 1621, 1622];
const IMAGE_WAVE_OVER: [u32; 2] = [1624, 1625];
const IMAGE_BEFORE_LAST: u32 = 1628;
const IMAGE_LOST: u32 = 1627;
const SCORE: u32 = 0x1000_000A;
const WORDS_GONE: u32 = 0x1000_000B;
const BETWEEN: u32 = 0x1000_000C;
const KILLS: u32 = 0x1000_000D;
const DEFEATED: u32 = 0x1000_0012;
const PHASE: u32 = 0x1000_0013;
const WAVE_CLEARED: u32 = 0x1000_0014;
const ENEMY_DOWN: u32 = 0x1000_0016;
/// The result (the two numbers retail sent with it), a special state's notice and its switch, and the lost game's.
const RESULT: u32 = 0x1000_0005;
const RESULT_ARGS: [u32; 2] = [12_782_296, 12_782_297];
const STATE_NOTICE: u32 = 0x1000_0011;
const STATE_SWITCH: u32 = 0x1000_0017;
const LOST: u32 = 0x1000_0018;
/// The music (the content director's own command) and the duty's end and its two time flags, as numbers.
const SET_BGM: u32 = 0x8000_0001;
const DUTY_COMPLETED: u32 = 0x4000_0002;
const DUTY_FLAG: u32 = 0x4000_0007;

/// `PHASE`'s values: between waves, the enemies appearing, the wave on, the game over.
const PHASE_BETWEEN: u32 = 0;
const PHASE_APPEAR: u32 = 1;
const PHASE_ON: u32 = 2;
const PHASE_OVER: u32 = 4;
const PHASE_LOST: u32 = 5;

/// `PLAYER`'s numbers: `[player, 12609, 12610]` once before the first wave, `[player, 708]` after each wave.
const PLAYER_AT_START: [u32; 2] = [12609, 12610];
const PLAYER_AFTER_WAVE: u32 = 708;
/// And `[player, 718, 3]` to each after a lost game.
const PLAYER_LOST: [u32; 2] = [718, 3];
/// `ENEMY_DOWN`'s second number (7,254 of 7,372; 3069 the rest.)
const ENEMY_DOWN_ARG: u32 = 3068;
/// `WORDS`: the list's ID is this plus the group, and the last number.
const WORD_LIST: u32 = 2_000_010;
const WORDS_LAST: u32 = 100;
/// The highest word of each group's list retail gave out in patch 7.57, less the three words 7.58 took out (so none
/// is past the end of 7.58's lists, wherever those three were.)
const WORD_COUNT: [u32; 3] = [425, 967, 280];

/// The barrier: its HP, its nine pieces (BNpcBase 19585, layout 12406198, z 17.5) and the tenth (20146, layout
/// 12743237, at (0, 0, 18)), all out for the whole run.
const BARRIER_HP: u32 = 40_000;
const BARRIER_BASE: u32 = 19585;
const BARRIER_LAYOUT: u32 = 12_406_198;
const BARRIER_X: [f32; 9] = [-9.0, -7.5, -4.5, -2.5, 0.0, 2.5, 4.5, 7.5, 9.0];
const BARRIER_Z: f32 = 17.5;
const BARRIER_MIDDLE: (u32, u32, f32) = (20146, 12_743_237, 18.0);
const BARRIER_PIECE_HP: u32 = 188_300;
/// The enemies' BNpcName (all but the last boss, whose is in the data) and level.
const ENEMY_NAME: u32 = 108;
const ENEMY_LEVEL: u32 = 100;

/// Where a marcher stops and starts hitting the barrier (retail's first blows: 35.5 yalms from z -24 at each kind's
/// speed, 12.0 s for 19587 and 16.0 s for 19586.)
const STOP_Z: f32 = 11.5;
/// Between an enemy's blows at the barrier, unless its data says otherwise.
const ATTACK_INTERVAL: f32 = 3.0;

/// Seconds: the first wave's enemies after the duty begins; a wave on after its enemies appear; the next wave's
/// enemies after a wave's end (and before the last wave); the two `BETWEEN`s after a wave's end.
const FIRST_APPEAR: f32 = 7.5;
const PLAYERS_AT: f32 = 2.0;
const WAVE_ON_AFTER: f32 = 5.0;
const NEXT_APPEAR: f32 = 12.6;
const LAST_APPEAR: f32 = 30.5;
const BETWEEN_AFTER: [f32; 2] = [3.0, 8.0];
/// The LogMessage with the first `BETWEEN` (0x2D2C.)
const LOG_WAVE_OVER: u32 = 11564;
/// An enemy getting its words is told 2.0 s before (ActorControl 407 `[7750]`), when it has waited.
const READY_BEFORE: f32 = 2.0;
const CONTROL_READY: (u32, u32) = (407, 7750);
/// ActorControl 54: `[1]` as an enemy gets its words, `[0]` as they go.
const CONTROL_WORDS: u32 = 54;
/// After an enemy's fall: its words gone, ActorControl 607, its DeleteActor.
const WORDS_GONE_AFTER: f32 = 2.0;
const FALLEN_CONTROL_AFTER: f32 = 3.04;
const CONTROL_FALLEN: u32 = 607;
const DELETE_AFTER: f32 = 4.32;

/// A hit's damage, and what a word is worth by its group (the barrier's own words, group 3: 400, 574 of them at
/// the first level.)
const HIT_DAMAGE: u32 = 100;
const WORD_POINTS: [u32; 4] = [100, 200, 300, 400];
/// The bonus gauge: full at this; a hit adds 15 times the word's worth; it drains by about this a second (586 to 702
/// in the clean stretches) from 0.535 s after a hit. Full, it goes round, and 1.2 s later it's set full again; going
/// round after that is a bonus level (each a fifth more a word, five at most.)
const GAUGE_FULL: f32 = 10_000.0;
const GAUGE_PER_POINT: f32 = 15.0;
const GAUGE_DRAIN: f32 = 620.0;
const GAUGE_HOLD: f32 = 0.535;
const GAUGE_REFILL_AFTER: f32 = 1.2;
const MOST_LEVELS: u32 = 5;
/// Impassioned Inputs (the client's "fever"): as a 19592 falls, for 20.0 s (67 of 67), every word is worth 1,000
/// (533 of 548; the rest 3,000, taken to be the big enemy's words), the gauge is held as it stood and the client
/// sends its words as 51027 and 51028. Its notice `[17835, 5342]` and switch `[3078, 1]` as it begins, `[17836, 5342]`
/// and `[3078, 0]` as it ends.
const FEVER_BASE: u32 = 19592;
const FEVER_SECONDS: f32 = 20.0;
const FEVER_POINTS: [u32; 3] = [1000, 3000, 3000];
const FEVER_NOTICE: [u32; 2] = [17835, 17836];
const FEVER_STATUS: u32 = 5342;
/// The status's parameter on each player (27 of 27.)
const FEVER_STATUS_PARAM: u16 = 1180;
const FEVER_SWITCH: u32 = 3078;
/// The scenes: before the last wave (cutscene 4024, then the music 20301) and the result (4025, 20302), both scene 3
/// of a GameProgress event with argument 1; the way out, scene 11 with argument 7 (as Air Force One's).
const SCENE: u16 = 3;
const CUTSCENE_FLAGS: u32 = 0x0850_2401;
const CUTSCENE_PARAMS: [u32; 5] = [4024, 3, 20301, 0, 0];
const RESULT_FLAGS: u32 = 0x0850_2403;
const RESULT_PARAMS: [u32; 5] = [4025, 1, 20302, 0, 0];
const SCENE_ARG: u32 = 1;
const LEAVE_SCENE: u16 = 11;
const LEAVE_FLAGS: u32 = 0x0C00_2001;
const LEAVE_ARG: u32 = 7;
/// The music: 1 as the cutscene nears and as a game is lost, then the last wave's and the lost game's.
const BGM_NONE: u32 = 1;
const BGM_LAST_WAVE: u32 = 20301;
const BGM_LOST: u32 = 20303;
/// Seconds after the second wave's end: the cutscene, and the last wave's music (retail's came as the cutscene ended,
/// 15.9 s after it began.)
const CUTSCENE_AT: f32 = 10.1;
const LAST_WAVE_BGM_AT: f32 = 26.0;
/// Seconds after a win: the result's numbers and the duty's completion. After a loss: the music and the players'
/// numbers, then the duty's end, the rewards and the way out.
const RESULT_AT: f32 = 8.05;
const LOST_AT: [f32; 2] = [3.05, 6.1];

/// A cast's effect lands this long after its bar ends (2.7 s casts landed 3.0 s after they began, 11.7 s ones 12.0.)
const CAST_LANDS_AFTER: f32 = 0.3;
/// One whose own action shows its fall (a bomb, a 19829) dies this long after it; a caster that falls by its own cast
/// the same.
const DIES_AFTER: f32 = 1.0;
/// An enemy that leaves without falling: ActorControl 607 and its DeleteActor come this much sooner than a felled
/// one's (its words go at once.)
const LEAVES_SOONER: f32 = WORDS_GONE_AFTER;
/// A summoned enemy appearing: ActorControl 36 `[0]` with its first action, then ActorControl 54 `[1]`.
const CONTROL_APPEAR: u32 = 36;
/// An action's effect of kind 27 (the action's own ID with it) and its fifth byte, as retail's had them.
const EFFECT_OWN: u8 = 27;
const EFFECT_FLAG: u8 = 0x80;

/// A wave's bonus for a whole barrier.
const WAVE_BONUS: u32 = 10_000;

/// The rewards: MGP and MGC a point of score, and the most of each (retail's 223 results fit to the unit: 2,759 MGP
/// and 347 MGC for 59,980; 2,800 and 350 from 60,870 up.)
const MGP_PER_POINT: f64 = 0.046;
const MGC_PER_POINT: f64 = 0.0058;
const MOST_MGP: u32 = 2_800;
const MOST_MGC: u32 = 350;
pub const ITEM_MGC: u32 = 52_322;
pub const ITEM_COFFER: u32 = 52_450;
/// The result's LogMessages: the MGP, the MGC, the coffer (0x2D4E, 0x2D51, 0x2D4B.)
const LOG_MGP: u32 = 11_598;
const LOG_MGC: u32 = 11_601;
const LOG_COFFER: u32 = 11_595;

/// What a score earns: MGP and MGC.
pub fn rewards(score: u32) -> (u32, u32) {
    (
        ((score as f64 * MGP_PER_POINT) as u32).min(MOST_MGP),
        ((score as f64 * MGC_PER_POINT) as u32).min(MOST_MGC),
    )
}

/// A kind of enemy (`keybound_waves.json`'s `enemies`.)
#[derive(Debug, Clone, PartialEq)]
pub struct EnemyKind {
    pub layout: u32,
    pub name: u32,
    /// With four players and with three (0: not seen.)
    pub hp: [u32; 2],
    pub speed: f32,
    pub stop: f32,
    pub action: u32,
    pub damage: u32,
    pub interval: f32,
    pub group: u32,
    /// The highest word it's given (the whole list unless the data says.)
    pub words: u32,
}

impl EnemyKind {
    /// Its HP for this many players. Retail's runs had four (220) or three (3): three had 100 to 1,800 fewer. Fewer
    /// than three weren't seen: each player fewer takes the same again off, down to one hit's worth.
    pub fn hp_for(&self, players: usize) -> u32 {
        let step = if self.hp[1] != 0 {
            self.hp[0] - self.hp[1]
        } else {
            // About 0.775 of it with three, to the hundred, as the others'.
            ((self.hp[0] as f32 * 0.225 / 100.0).round() as u32).max(1) * 100
        };
        let fewer = 4usize.saturating_sub(players.clamp(1, 4)) as u32;
        self.hp[0].saturating_sub(step * fewer).max(HIT_DAMAGE)
    }
}

/// One enemy of a line-up: its kind, where, when its SpawnNpc comes and when it sets off (seconds from the wave's
/// enemies appearing.)
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub base: u32,
    pub x: f32,
    pub z: f32,
    pub spawn: f32,
    pub active: f32,
}

#[derive(Debug, Default)]
pub struct Wave {
    pub sets: Vec<Vec<Entry>>,
    /// The big enemies one of which the wave has (`[base, x, z]`), and when it gets its words.
    pub bosses: Vec<(u32, f32, f32)>,
    pub boss_active: f32,
}

#[derive(Debug, Default)]
pub struct Waves {
    pub kills_to_clear_wave_1: u32,
    pub waves: Vec<Wave>,
    pub enemies: HashMap<u32, EnemyKind>,
}

/// The waves and enemies (`resources/data/keybound_waves.json`.)
pub fn waves() -> &'static Waves {
    static WAVES: OnceLock<Waves> = OnceLock::new();
    WAVES.get_or_init(|| {
        let value: Value = serde_json::from_str(include_str!(
            "../../../../resources/data/keybound_waves.json"
        ))
        .expect("resources/data/keybound_waves.json doesn't read");
        let number = |value: &Value| value.as_f64().unwrap_or(0.0) as f32;
        let enemies = value["enemies"]
            .as_object()
            .map(|kinds| {
                kinds
                    .iter()
                    .filter_map(|(base, kind)| {
                        let group = kind["group"].as_u64().unwrap_or(0) as u32;
                        Some((
                            base.parse().ok()?,
                            EnemyKind {
                                layout: kind["layout"].as_u64()? as u32,
                                name: kind["name"].as_u64().unwrap_or(ENEMY_NAME as u64) as u32,
                                hp: [
                                    kind["hp"][0].as_u64()? as u32,
                                    kind["hp"][1].as_u64().unwrap_or(0) as u32,
                                ],
                                speed: number(&kind["speed"]),
                                stop: kind["stop"].as_f64().map(|z| z as f32).unwrap_or(STOP_Z),
                                action: kind["action"].as_u64().unwrap_or(0) as u32,
                                damage: kind["damage"].as_u64().unwrap_or(0) as u32,
                                interval: kind["interval"]
                                    .as_f64()
                                    .map(|s| s as f32)
                                    .unwrap_or(ATTACK_INTERVAL),
                                group,
                                words: kind["words"].as_u64().map(|w| w as u32 + 1).unwrap_or(
                                    WORD_COUNT[(group as usize).min(WORD_COUNT.len() - 1)],
                                ),
                            },
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let waves = value["waves"]
            .as_array()
            .map(|waves| {
                waves
                    .iter()
                    .map(|wave| Wave {
                        sets: wave["sets"]
                            .as_array()
                            .map(|sets| {
                                sets.iter()
                                    .map(|groups| {
                                        groups
                                            .as_array()
                                            .into_iter()
                                            .flatten()
                                            .flat_map(|group| {
                                                group.as_array().into_iter().flatten()
                                            })
                                            .map(|entry| Entry {
                                                base: entry[0].as_u64().unwrap_or(0) as u32,
                                                x: number(&entry[1]),
                                                z: number(&entry[2]),
                                                spawn: number(&entry[3]),
                                                active: number(&entry[4]),
                                            })
                                            .collect()
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        bosses: wave["bosses"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .map(|boss| {
                                (
                                    boss[0].as_u64().unwrap_or(0) as u32,
                                    number(&boss[1]),
                                    number(&boss[2]),
                                )
                            })
                            .collect(),
                        boss_active: number(&wave["boss_active"]),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Waves {
            kills_to_clear_wave_1: value["kills_to_clear_wave_1"].as_u64().unwrap_or(25) as u32,
            waves,
            enemies,
        }
    })
}

/// A player's bonus gauge and score.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bonus {
    pub score: u32,
    pub gauge: f32,
    pub level: u32,
    /// The gauge went round and was set full again: going round once more is a level.
    pub armed: bool,
}

impl Bonus {
    /// The gauge after `idle` seconds without a hit (it holds for `GAUGE_HOLD`, then drains.)
    pub fn drained(&self, idle: f32) -> f32 {
        (self.gauge - GAUGE_DRAIN * (idle - GAUGE_HOLD).max(0.0)).max(0.0)
    }

    /// A word of `group` hit, `idle` seconds after the last: the score it adds, and whether the gauge went round
    /// without a level (so it's set full again a moment later, `refill`.)
    pub fn hit(&mut self, group: u32, idle: f32) -> (u32, bool) {
        let worth = WORD_POINTS[(group as usize).min(WORD_POINTS.len() - 1)];
        // The word scores at the level the player had as it landed.
        let points = worth * (5 + self.level) / 5;
        self.score += points;
        self.gauge = self.drained(idle) + GAUGE_PER_POINT * worth as f32;
        let mut round = false;
        if self.gauge >= GAUGE_FULL {
            self.gauge -= GAUGE_FULL;
            if self.armed {
                self.armed = false;
                self.level = (self.level + 1).min(MOST_LEVELS);
            } else {
                round = true;
            }
        }
        (points, round)
    }

    /// A word under Impassioned Inputs: ten times its worth, whatever the level, and the gauge stays as it is.
    pub fn fever_hit(&mut self, group: u32) -> u32 {
        let points = FEVER_POINTS[(group as usize).min(FEVER_POINTS.len() - 1)];
        self.score += points;
        points
    }

    /// The gauge set full after going round.
    pub fn refill(&mut self) {
        self.gauge = GAUGE_FULL;
        self.armed = true;
    }

    /// A wave over: the bonus for the barrier's HP, and the gauge and level begin again.
    pub fn wave_over(&mut self, barrier_hp: u32) -> u32 {
        let bonus = (WAVE_BONUS as u64 * barrier_hp as u64 / BARRIER_HP as u64) as u32;
        self.score += bonus;
        self.gauge = 0.0;
        self.level = 0;
        self.armed = false;
        bonus
    }
}

#[derive(Debug)]
struct Player {
    bonus: Bonus,
    last_hit: Option<Instant>,
    /// When the client is told the gauge drains (0.535 s after a hit), and when it's set full again after going round.
    drain_at: Option<Instant>,
    refill_at: Option<Instant>,
}

#[derive(Debug)]
struct Enemy {
    base: u32,
    hp: u32,
    /// Its word and the one after (the client shows both.)
    words: Option<(u32, u32)>,
    /// When it gets its words and sets off; None once it has.
    active_at: Option<Instant>,
    ready_sent: bool,
    arrived: bool,
    next_attack: Option<Instant>,
    /// Its wave's big enemy: its fall ends the wave.
    boss: bool,
    fallen: Option<Instant>,
    /// What's been sent since its fall: its words gone, ActorControl 607.
    after_fall: u8,
    /// Its SpawnNpc, and what's been done of a summoned enemy's first moments (its appearing, its first blow.)
    spawned_at: Instant,
    appeared: bool,
    hit: bool,
    /// The cast it begins once it has its words, and when; the cast it's in.
    to_cast: Option<(Spell, Then, Instant)>,
    casting: Option<Casting>,
    /// A big enemy's rotation: when it begins, then where it stands.
    script_at: Option<Instant>,
    script: Option<Place>,
    /// Its own action showed its fall: it dies then.
    dying: Option<Instant>,
    /// The barrier's words: filled to `full` by words (each player has their own.)
    full: u32,
    own_words: HashMap<ObjectId, (u32, u32)>,
}

/// A cast under way.
#[derive(Debug, Clone)]
struct Casting {
    spell: Spell,
    then: Then,
    begun: Instant,
    /// The counter's stacks left.
    stacks: u32,
}

/// What's still to come at a set time.
#[derive(Debug, Clone)]
enum Pending {
    /// A blow at the barrier from the helper (or whoever cast it.)
    Blow {
        from: ObjectId,
        action: u32,
        damage: u32,
    },
    StateOff {
        notice: u32,
        arg: u32,
        switch: u32,
    },
    /// Every player's warning status, a player's lock and its end.
    Warn {
        status: u16,
    },
    Lock {
        player: ObjectId,
        param: u16,
    },
    Unlock {
        player: ObjectId,
    },
    /// Another kind where one stood.
    Become {
        base: u32,
        x: f32,
        z: f32,
    },
}

/// A status given or taken, done once the network isn't held.
#[derive(Debug, Clone)]
enum StatusChange {
    Gain(ObjectId, u16, u16, f32),
    Lose(ObjectId, u16),
}

impl Enemy {
    fn new(base: u32, hp: u32, now: Instant) -> Self {
        Self {
            base,
            hp,
            words: None,
            active_at: None,
            ready_sent: true,
            arrived: false,
            next_attack: None,
            boss: false,
            fallen: None,
            after_fall: 0,
            spawned_at: now,
            appeared: false,
            hit: false,
            to_cast: None,
            casting: None,
            script_at: None,
            script: None,
            dying: None,
            full: 0,
            own_words: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum Stage {
    /// Before the first wave: the players' numbers, then the enemies.
    Start { players_sent: bool },
    /// A wave's enemies come and the wave runs; `on` once `WAVE_ON` went out.
    Wave { on: bool },
    /// After a wave: the two `BETWEEN`s, then the next.
    Between { sent: u8 },
    /// The game over, won or lost: what's still to come (`step` counts what's been sent), then the players leave from
    /// their result.
    Ending { won: bool, step: u8 },
}

/// A run of the game.
#[derive(Debug)]
pub struct Game {
    director: HandlerId,
    stage: Stage,
    /// The wave (0 to 2), its line-up and which of its entries have spawned.
    wave: usize,
    entries: Vec<Entry>,
    spawned: usize,
    /// When the stage's clock is 0: the duty's start, the wave's enemies appearing, the wave's end.
    clock: Instant,
    last_turn: Instant,
    barrier_hp: u32,
    barrier: Vec<ObjectId>,
    enemies: HashMap<ObjectId, Enemy>,
    players: HashMap<ObjectId, Player>,
    kills: u32,
    /// How many players the enemies' HP is for (those in as the game began.)
    party: usize,
    /// Whether the wave's `APPEAR` has gone out.
    appear_sent: bool,
    /// Impassioned Inputs, until then.
    fever_until: Option<Instant>,
    /// Before the last wave: the cutscene played, its music set.
    cutscene: u8,
    /// The barrier's shield is up (its words were filled.)
    shield: bool,
    /// The barrier's words on the field.
    barrier_words: Option<ObjectId>,
    /// The helper the big enemies' blows come from.
    helper: Option<ObjectId>,
    pending: Vec<(Instant, Pending)>,
    statuses: Vec<StatusChange>,
    /// A cast that warns is under way or just over: until when a client may ask for its lock, and the lock's
    /// parameter.
    warned: Option<(Instant, u16)>,
    locked: Vec<ObjectId>,
    /// Kinds to take another's place this turn: `(base, x, z)`.
    becoming: Vec<(u32, f32, f32)>,
    /// The players told who's in the game.
    listed: Vec<ObjectId>,
    /// The duty has commenced and its first phase gone out.
    begun: bool,
}

fn elapsed(since: Instant, now: Instant) -> f32 {
    now.saturating_duration_since(since).as_secs_f32()
}

fn seconds(s: f32) -> Duration {
    Duration::from_secs_f32(s.max(0.0))
}

/// The players in the duty (not bots.)
fn players_in(instance: &Instance) -> Vec<ObjectId> {
    instance
        .actors
        .iter()
        .filter(|(id, actor)| {
            matches!(actor, NetworkedActor::Player { .. })
                && !instance.party_bots.contains_key(id)
                && !instance.world_bots.contains(id)
        })
        .map(|(id, _)| *id)
        .collect()
}

/// A director command to one player.
fn command(network: &mut NetworkState, director: HandlerId, to: ObjectId, id: u32, args: &[u32]) {
    let mut seven = [0u32; 7];
    for (slot, value) in seven.iter_mut().zip(args) {
        *slot = *value;
    }
    network.send_to_by_actor_id(
        to,
        FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
            handler_id: director,
            event: DirectorEvent::Raw { id, args: seven },
        }),
        DestinationNetwork::ZoneClients,
    );
}

/// A director command to every player.
fn command_all(
    network: &mut NetworkState,
    instance: &Instance,
    director: HandlerId,
    id: u32,
    args: &[u32],
) {
    for player in players_in(instance) {
        command(network, director, player, id, args);
    }
}

/// An actor's ActorControl with these numbers, to everyone.
fn control(
    network: &mut NetworkState,
    instance: &Instance,
    actor: ObjectId,
    category: u32,
    params: [u32; 4],
) {
    network.send_ac_in_range_instance(
        instance,
        actor,
        ActorControlCategory::Unknown {
            category,
            param1: params[0],
            param2: params[1],
            param3: params[2],
            param4: params[3],
            param5: 0,
        },
    );
}

/// One actor's action on another for this much, as everyone sees it.
fn blow(
    network: &mut NetworkState,
    instance: &Instance,
    from: ObjectId,
    action: u32,
    target: ObjectId,
    damage: u32,
    sequence: u16,
) {
    act(
        network,
        instance,
        from,
        action,
        Some(target),
        TargetEffectKind::Damage {
            damage_kind: DamageKind::default(),
            damage_type: DamageType::Unique,
            damage_element: DamageElement::Unaspected,
            bonus_percent: 0,
            unk3: 0,
            unk4: 0,
            amount: damage.min(u16::MAX as u32) as u16,
        },
        sequence,
    );
}

/// An action's effect of this kind with the action's own ID (retail's kind 27 on the enemies' own actions, 72 on the
/// bombs' appearing.)
fn own_effect(kind: u8, action: u32) -> TargetEffectKind {
    match kind {
        0 => TargetEffectKind::None,
        1 => TargetEffectKind::Miss,
        EFFECT_OWN => TargetEffectKind::ExecuteCombo {
            sequence: 0,
            unk2: 0,
            unk3: 0,
            unk4: 0,
            unk5: EFFECT_FLAG,
            action_id: action as u16,
        },
        // The bombs' 49534: `48 01 00 00 00 80 00 00`.
        _ => TargetEffectKind::Unknown {
            magic: kind,
            param0: 1,
            param1: 0,
            param2: 0,
            param3: 0,
            param4: EFFECT_FLAG,
            value: 0,
        },
    }
}

/// One actor's action with this effect on a target (or on no one), as everyone sees it.
fn act(
    network: &mut NetworkState,
    instance: &Instance,
    from: ObjectId,
    action: u32,
    target: Option<ObjectId>,
    effect: TargetEffectKind,
    sequence: u16,
) {
    let rotation = instance
        .find_actor(from)
        .map(|actor| actor.rotation())
        .unwrap_or(0.0);
    let targets: Vec<ObjectTypeId> = target
        .into_iter()
        .map(|object_id| ObjectTypeId {
            object_id,
            object_type: ObjectTypeKind::None,
        })
        .collect();
    let mut effects = [TargetEffect::default(); 8];
    effects[0] = TargetEffect(effect);
    let ipc = ActionEffect {
        animation_target_id: targets.first().copied().unwrap_or(ObjectTypeId {
            object_id: from,
            object_type: ObjectTypeKind::None,
        }),
        targets,
        action_id: action,
        animation_lock: 0.11,
        rotation,
        spell_id: action as u16,
        source_sequence: sequence,
        action_type: ActionType::Action,
        global_sequence: network.global_action_sequence,
        effects,
        ..Default::default()
    }
    .package()
    .unwrap();
    network.global_action_sequence += 1;
    network.send_to_instance(
        ObjectId::default(),
        instance,
        FromServer::PacketSegment(ipc, from),
        DestinationNetwork::ZoneClients,
    );
}

/// One of the duty's map effects, to everyone.
fn map_effect(
    network: &mut NetworkState,
    instance: &Instance,
    director: HandlerId,
    (state, timeline_id, index): (u16, u16, u8),
) {
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::DirectorMapEffect {
        handler_id: director,
        state,
        timeline_id,
        index,
    });
    network.send_to_instance(
        ObjectId::default(),
        instance,
        FromServer::PacketSegment(ipc, ObjectId::default()),
        DestinationNetwork::ZoneClients,
    );
}

/// A banner, to everyone (`IMAGE_WAVE`.)
fn banner(network: &mut NetworkState, instance: &Instance, image: u32) {
    for player in players_in(instance) {
        network.send_to_by_actor_id(
            player,
            FromServer::ActorControlSelf(ActorControlCategory::ShowScreenImage {
                screen_image: image,
                target_actor_id: ObjectId::default(),
            }),
            DestinationNetwork::ZoneClients,
        );
    }
}

/// A player's bonus gauge as the client is told it (`KeyboundGauge`): held, or draining from now.
fn send_gauge(network: &mut NetworkState, to: ObjectId, gauge: f32, held: bool) {
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::KeyboundGauge {
        held: held as u32,
        gauge: gauge.clamp(0.0, GAUGE_FULL) as u32,
        time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_millis() as u64)
            .unwrap_or(0),
    });
    network.send_to_by_actor_id(
        to,
        FromServer::PacketSegment(ipc, to),
        DestinationNetwork::ZoneClients,
    );
}

/// One of the director's scenes for a player, in a GameProgress event with this argument.
fn play_scene(
    network: &mut NetworkState,
    director: HandlerId,
    to: ObjectId,
    scene: u16,
    flags: u32,
    params: &[u32],
    arg: u32,
) {
    network.send_to_by_actor_id(
        to,
        FromServer::PlayDirectorSceneWith {
            handler: director,
            scene,
            flags,
            params: params.to_vec(),
            arg,
        },
        DestinationNetwork::ZoneClients,
    );
}

/// The four places of the result: each player's place (1 the highest score), score, content ID and account ID, in
/// the order of their content IDs (retail's; places without a player are zeros.)
pub fn result_places(mut players: Vec<(u64, u64, u32)>) -> ([u8; 4], [u32; 4], [u64; 4], [u64; 4]) {
    players.sort();
    players.truncate(4);
    let mut places = [0u8; 4];
    let mut scores = [0u32; 4];
    let mut content_ids = [0u64; 4];
    let mut account_ids = [0u64; 4];
    for (index, (content_id, account_id, score)) in players.iter().enumerate() {
        places[index] = 1 + players.iter().filter(|other| other.2 > *score).count() as u8;
        scores[index] = *score;
        content_ids[index] = *content_id;
        account_ids[index] = *account_id;
    }
    (places, scores, content_ids, account_ids)
}

impl Game {
    fn new(director: HandlerId, now: Instant, party: usize) -> Self {
        Self {
            director,
            stage: Stage::Start {
                players_sent: false,
            },
            wave: 0,
            entries: Vec::new(),
            spawned: 0,
            clock: now,
            last_turn: now,
            barrier_hp: BARRIER_HP,
            barrier: Vec::new(),
            enemies: HashMap::new(),
            players: HashMap::new(),
            kills: 0,
            party: party.max(1),
            appear_sent: false,
            fever_until: None,
            cutscene: 0,
            shield: false,
            barrier_words: None,
            helper: None,
            pending: Vec::new(),
            statuses: Vec::new(),
            warned: None,
            locked: Vec::new(),
            becoming: Vec::new(),
            listed: Vec::new(),
            begun: false,
        }
    }

    /// Each player not yet told gets who's in the game (their own content ID first), and those already told the
    /// newcomer.
    fn list_players(&mut self, network: &mut NetworkState, instance: &Instance) -> Vec<ObjectId> {
        // A player still loading has no content ID here yet (theirs went out as 0, 0.)
        let content_id = |player: ObjectId| match instance.find_actor(player) {
            Some(NetworkedActor::Player { spawn, .. }) if spawn.content_id != 0 => {
                Some(spawn.content_id)
            }
            _ => None,
        };
        let mut new = Vec::new();
        let players = players_in(instance);
        for player in &players {
            if self.listed.contains(player) {
                continue;
            }
            let Some(own) = content_id(*player) else {
                continue;
            };
            let entry = |id: u64| [(id >> 32) as u32, id as u32];
            command(network, self.director, *player, PLAYERS, &entry(own));
            for other in &players {
                if other == player {
                    continue;
                }
                if let Some(id) = content_id(*other) {
                    command(network, self.director, *player, PLAYERS, &entry(id));
                }
                if self.listed.contains(other) {
                    command(network, self.director, *other, PLAYERS, &entry(own));
                }
            }
            self.listed.push(*player);
            new.push(*player);
        }
        new
    }

    /// The wave's line-up: one of its sets and, with a big enemy, one of those (`pick` gives a number below its
    /// argument.)
    fn line_up(wave: &Wave, mut pick: impl FnMut(usize) -> usize) -> Vec<Entry> {
        let mut entries = if wave.sets.is_empty() {
            Vec::new()
        } else {
            wave.sets[pick(wave.sets.len())].clone()
        };
        if !wave.bosses.is_empty() {
            let (base, x, z) = wave.bosses[pick(wave.bosses.len())];
            entries.push(Entry {
                base,
                x,
                z,
                // Wave 2's came with the first enemies; the last boss 20.2 s before its wave, during the cutscene.
                spawn: -4.3,
                active: wave.boss_active,
            });
        }
        entries.sort_by(|a, b| a.spawn.total_cmp(&b.spawn));
        entries
    }

    /// A word nobody on the field carries (retail never showed one twice at once.)
    fn free_word(&self, group: u32, most: u32, mut pick: impl FnMut(u32) -> u32) -> u32 {
        let taken: Vec<u32> = self
            .enemies
            .values()
            .filter(|enemy| {
                waves()
                    .enemies
                    .get(&enemy.base)
                    .is_some_and(|kind| kind.group == group)
            })
            .flat_map(|enemy| enemy.words.into_iter().flat_map(|(a, b)| [a, b]))
            .collect();
        let most = most.max(1);
        for _ in 0..64 {
            let word = pick(most);
            if !taken.contains(&word) {
                return word;
            }
        }
        (0..most).find(|word| !taken.contains(word)).unwrap_or(0)
    }
}

fn spawn_piece(
    instance: &mut Instance,
    game_data: &mut GameData,
    director: HandlerId,
    base: u32,
    name: u32,
    layout: u32,
    hp: u32,
    hp_now: u32,
    position: Position,
) -> Option<ObjectId> {
    // A game install without the event's rows (before patch 7.57) can't show it.
    game_data.find_bnpc(base)?;
    let mut npc = create_npc_common_spawn(game_data, base, name, Some(hp), ENEMY_LEVEL);
    npc.common.layout_id = layout;
    npc.common.handler_id = director;
    npc.common.position = position;
    npc.common.rotation = 0.0;
    npc.common.max_health_points = hp;
    npc.common.health_points = hp_now;
    let actor_id = Instance::generate_actor_id();
    instance.insert_npc(actor_id, npc, &get_config(), game_data);
    // The game moves it and has it hit the barrier; it has no fight of its own.
    instance.held_npcs.insert(actor_id);
    Some(actor_id)
}

/// The world's turn for a Keybound Brawler duty.
pub fn tick(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    game_data: Arc<Mutex<GameData>>,
) {
    if instance.content_finder_condition_id != DUTY {
        return;
    }
    let now = Instant::now();
    let Some(director) = instance.directors.first().map(|director| director.id) else {
        return;
    };

    // The barrier is out from the start; the game begins with the duty (everyone's opening over.)
    if instance.keybound.is_none() {
        let players = players_in(instance);
        if players.is_empty() {
            return;
        }
        let mut game = Game::new(director, now, players.len());
        {
            let mut game_data = game_data.lock();
            for x in BARRIER_X {
                if let Some(id) = spawn_piece(
                    instance,
                    &mut game_data,
                    director,
                    BARRIER_BASE,
                    ENEMY_NAME,
                    BARRIER_LAYOUT,
                    BARRIER_PIECE_HP,
                    BARRIER_PIECE_HP,
                    Position(glam::Vec3A::new(x, 0.0, BARRIER_Z)),
                ) {
                    game.barrier.push(id);
                }
            }
            let (base, layout, z) = BARRIER_MIDDLE;
            if let Some(id) = spawn_piece(
                instance,
                &mut game_data,
                director,
                base,
                ENEMY_NAME,
                layout,
                BARRIER_PIECE_HP,
                BARRIER_PIECE_HP,
                Position(glam::Vec3A::new(0.0, 0.0, z)),
            ) {
                game.barrier.push(id);
            }
        }
        if game.barrier.is_empty() {
            tracing::warn!(
                "Keybound Brawler: the game data has no BNpcBase {BARRIER_BASE}; the duty can't be played"
            );
        }
        instance.keybound = Some(game);
        if let Some(director) = instance.directors.first_mut() {
            director.flag = SEQUENCE;
        }
    }
    // Who's in the game, to each player as they come (`PLAYERS`), and the game's status on them.
    if let Some(mut game) = instance.keybound.take() {
        let new = game.list_players(&mut network.lock(), instance);
        instance.keybound = Some(game);
        for player in new {
            gain_effect_instance(
                network.clone(),
                ClientId::default(),
                instance,
                player,
                PLAYER_STATUS.0,
                PLAYER_STATUS.1,
                0.0,
                ObjectId::default(),
                false,
            );
            crate::server::effect::process_effects_list(network.clone(), instance, player);
        }
    }
    if instance.awaiting_commence {
        if let Some(game) = instance.keybound.as_mut() {
            game.clock = now;
            game.last_turn = now;
        }
        return;
    }

    let Some(mut game) = instance.keybound.take() else {
        return;
    };
    game.turn(&network, instance, &game_data, now);
    instance.keybound = Some(game);
}

impl Game {
    fn turn(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        game_data: &Arc<Mutex<GameData>>,
        now: Instant,
    ) {
        let step = elapsed(self.last_turn, now);
        self.last_turn = now;
        let clock = elapsed(self.clock, now);
        for player in players_in(instance) {
            self.players.entry(player).or_insert(Player {
                bonus: Bonus::default(),
                last_hit: None,
                drain_at: None,
                refill_at: None,
            });
        }
        self.gauges(&mut network.lock(), instance, now);
        self.fallen(network, instance, now);
        self.timed(network, instance, now);
        for (base, x, z) in std::mem::take(&mut self.becoming) {
            self.summon(instance, game_data, base, x, z, now);
        }

        match self.stage {
            Stage::Start { players_sent } => {
                // As the duty commences, with its timer (223 of 223: `0x10000013 [0]`.)
                if !self.begun {
                    self.begun = true;
                    command_all(&mut network.lock(), instance, self.director, PHASE, &[0]);
                }
                if !players_sent && clock >= PLAYERS_AT {
                    let mut network = network.lock();
                    for player in players_in(instance) {
                        command_all(
                            &mut network,
                            instance,
                            self.director,
                            PLAYER,
                            &[player.0, PLAYER_AT_START[0], PLAYER_AT_START[1]],
                        );
                    }
                    banner(&mut network, instance, IMAGE_WAVE[0]);
                    self.stage = Stage::Start { players_sent: true };
                }
                // The first enemies' SpawnNpcs come up to 4.2 s before they appear.
                if clock >= FIRST_APPEAR - 4.5 {
                    self.begin_wave(0, now + seconds(FIRST_APPEAR - clock));
                }
            }
            Stage::Wave { on } => {
                // The stage's clock runs from the enemies appearing, so it's negative before.
                let wave_clock = if now >= self.clock {
                    clock
                } else {
                    -elapsed(now, self.clock)
                };
                self.spawns(instance, game_data, wave_clock);
                if !on && wave_clock >= 0.0 && !self.appear_sent {
                    let mut network = network.lock();
                    command_all(&mut network, instance, self.director, APPEAR, &[]);
                    command_all(
                        &mut network,
                        instance,
                        self.director,
                        PHASE,
                        &[PHASE_APPEAR],
                    );
                    self.appear_sent = true;
                }
                if !on && wave_clock >= WAVE_ON_AFTER {
                    let mut network = network.lock();
                    command_all(&mut network, instance, self.director, WAVE_ON, &[]);
                    command_all(&mut network, instance, self.director, PHASE, &[PHASE_ON]);
                    self.stage = Stage::Wave { on: true };
                }
                self.enemies_turn(network, instance, game_data, now, step);
                if self.barrier_hp == 0 {
                    self.finish(network, instance, false, now);
                } else if matches!(self.stage, Stage::Wave { on: true }) && self.wave_cleared() {
                    self.end_wave(network, instance, now);
                }
            }
            Stage::Between { sent } => {
                if (sent as usize) < BETWEEN_AFTER.len() && clock >= BETWEEN_AFTER[sent as usize] {
                    let mut network = network.lock();
                    command_all(&mut network, instance, self.director, BETWEEN, &[]);
                    if sent == 1 {
                        banner(
                            &mut network,
                            instance,
                            if self.wave == 0 {
                                IMAGE_WAVE[1]
                            } else {
                                IMAGE_BEFORE_LAST
                            },
                        );
                    }
                    if sent == 0 {
                        let ipc = EventLogMessage {
                            handler_id: self.director,
                            message_type: LOG_WAVE_OVER,
                            params: vec![0],
                        }
                        .package()
                        .unwrap();
                        network.send_to_instance(
                            ObjectId::default(),
                            instance,
                            FromServer::PacketSegment(ipc, ObjectId::default()),
                            DestinationNetwork::ZoneClients,
                        );
                        // Whole again, and looking it (retail's map effect came only when it didn't.)
                        if barrier_look(self.barrier_hp) != 0 {
                            map_effect(&mut network, instance, self.director, MAP_LOOK[0]);
                        }
                        self.barrier_hp = BARRIER_HP;
                        command_all(
                            &mut network,
                            instance,
                            self.director,
                            BARRIER_SET,
                            &[BARRIER_HP],
                        );
                        for player in players_in(instance) {
                            command_all(
                                &mut network,
                                instance,
                                self.director,
                                PLAYER,
                                &[player.0, PLAYER_AFTER_WAVE],
                            );
                        }
                    }
                    self.stage = Stage::Between { sent: sent + 1 };
                }
                let before_last = self.wave + 1 == waves().waves.len() - 1;
                if before_last {
                    // The music stops with the second `BETWEEN`, the cutscene plays, and the last wave's music
                    // begins.
                    let mut network = network.lock();
                    if self.cutscene == 0 && clock >= BETWEEN_AFTER[1] {
                        self.cutscene = 1;
                        command_all(&mut network, instance, self.director, SET_BGM, &[BGM_NONE]);
                    }
                    if self.cutscene == 1 && clock >= CUTSCENE_AT {
                        self.cutscene = 2;
                        for player in players_in(instance) {
                            play_scene(
                                &mut network,
                                self.director,
                                player,
                                SCENE,
                                CUTSCENE_FLAGS,
                                &CUTSCENE_PARAMS,
                                SCENE_ARG,
                            );
                        }
                    }
                    if self.cutscene == 2 && clock >= LAST_WAVE_BGM_AT {
                        self.cutscene = 3;
                        banner(&mut network, instance, IMAGE_WAVE[2]);
                        command_all(
                            &mut network,
                            instance,
                            self.director,
                            SET_BGM,
                            &[BGM_LAST_WAVE],
                        );
                    }
                }
                let appear = if before_last {
                    LAST_APPEAR
                } else {
                    NEXT_APPEAR
                };
                if clock >= appear - 4.5 {
                    self.begin_wave(self.wave + 1, now + seconds(appear - clock));
                }
            }
            Stage::Ending { won, step } => self.ending(network, instance, won, step, clock),
        }
        self.give_statuses(network, instance);
    }

    fn begin_wave(&mut self, wave: usize, appear_at: Instant) {
        let Some(data) = waves().waves.get(wave) else {
            return;
        };
        self.wave = wave;
        self.entries = Self::line_up(data, |below| fastrand::usize(..below));
        self.spawned = 0;
        self.kills = 0;
        self.clock = appear_at;
        self.appear_sent = false;
        self.stage = Stage::Wave { on: false };
    }

    /// The line-up's enemies whose SpawnNpc is due.
    fn spawns(
        &mut self,
        instance: &mut Instance,
        game_data: &Arc<Mutex<GameData>>,
        wave_clock: f32,
    ) {
        while self.spawned < self.entries.len() && self.entries[self.spawned].spawn <= wave_clock {
            let entry = self.entries[self.spawned].clone();
            self.spawned += 1;
            let Some(kind) = waves().enemies.get(&entry.base) else {
                continue;
            };
            let hp = kind.hp_for(self.party);
            let Some(id) = spawn_piece(
                instance,
                &mut game_data.lock(),
                self.director,
                entry.base,
                kind.name,
                kind.layout,
                hp,
                hp,
                Position(glam::Vec3A::new(entry.x, 0.0, entry.z)),
            ) else {
                continue;
            };
            let boss = waves().waves[self.wave]
                .bosses
                .iter()
                .any(|(base, _, _)| *base == entry.base);
            self.enemies.insert(
                id,
                Enemy {
                    base: entry.base,
                    hp,
                    words: None,
                    active_at: Some(self.clock + seconds(entry.active.max(0.0))),
                    // The first to set off, as the wave begins, weren't told beforehand.
                    ready_sent: entry.active <= WAVE_ON_AFTER,
                    arrived: false,
                    next_attack: None,
                    boss,
                    ..Enemy::new(entry.base, hp, Instant::now())
                },
            );
        }
    }

    /// The first wave is over at its count of kills; the others as their big enemy falls.
    fn wave_cleared(&self) -> bool {
        if self.wave == 0 {
            self.kills >= waves().kills_to_clear_wave_1
        } else {
            self.kills >= 1
        }
    }

    /// The enemies' turn: words for those setting off, the march, the blows at the barrier.
    fn enemies_turn(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        game_data: &Arc<Mutex<GameData>>,
        now: Instant,
        step: f32,
    ) {
        let ids: Vec<ObjectId> = self.enemies.keys().copied().collect();
        for id in ids {
            let Some(kind) = self
                .enemies
                .get(&id)
                .and_then(|enemy| waves().enemies.get(&enemy.base))
            else {
                continue;
            };
            let (active_at, ready_sent, fallen) = {
                let enemy = &self.enemies[&id];
                (enemy.active_at, enemy.ready_sent, enemy.fallen.is_some())
            };
            if fallen {
                continue;
            }
            self.first_moments(network, instance, id, now);
            if let Some(at) = active_at {
                if !ready_sent && now + seconds(READY_BEFORE) >= at {
                    control(
                        &mut network.lock(),
                        instance,
                        id,
                        CONTROL_READY.0,
                        [CONTROL_READY.1, 0, 0, 0],
                    );
                    self.enemies.get_mut(&id).unwrap().ready_sent = true;
                }
                if now < at {
                    continue;
                }
                let first = self.free_word(kind.group, kind.words, |most| fastrand::u32(..most));
                let enemy = self.enemies.get_mut(&id).unwrap();
                enemy.words = Some((first, first));
                let second = self.free_word(kind.group, kind.words, |most| fastrand::u32(..most));
                let enemy = self.enemies.get_mut(&id).unwrap();
                enemy.words = Some((first, second));
                enemy.active_at = None;
                self.set_off(network, instance, game_data, id, now);
            }

            // Its cast and its rotation; one casting neither marches nor hits.
            self.casts(network, instance, id, now);
            self.rotation(network, instance, game_data, id, now, step);
            match self.enemies.get(&id) {
                Some(enemy) if enemy.fallen.is_none() => {
                    if enemy.casting.is_some() || enemy.to_cast.is_some() {
                        continue;
                    }
                }
                _ => continue,
            }

            // The march, straight at the barrier.
            let arrived = self.enemies[&id].arrived;
            if !arrived {
                let Some(position) = instance.find_actor(id).map(|actor| actor.position().0) else {
                    continue;
                };
                if kind.speed <= 0.0 || position.z >= kind.stop {
                    let enemy = self.enemies.get_mut(&id).unwrap();
                    enemy.arrived = true;
                    enemy.next_attack = Some(now);
                    if kind.speed > 0.0 && self.arrived(network, instance, id, now) {
                        continue;
                    }
                } else {
                    let z = (position.z + kind.speed * step).min(kind.stop);
                    let to = Position(glam::Vec3A::new(position.x, position.y, z));
                    if let Some(NetworkedActor::Npc {
                        spawn,
                        last_position,
                        ..
                    }) = instance.find_actor_mut(id)
                    {
                        spawn.common.position = to;
                        *last_position = Some(to.0);
                    }
                    network.lock().send_in_range_instance(
                        id,
                        instance,
                        FromServer::ActorMove(
                            id,
                            to,
                            0.0,
                            kawari::common::MoveAnimationType::empty(),
                            kawari::common::MoveAnimationState::None,
                            kawari::common::JumpState::NoneOrFalling,
                            // The speed byte is tenths of a yalm a second, as `npc_behavior` sends it.
                            Some((kind.speed * 10.0) as u8),
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                    continue;
                }
            }

            // At the barrier: a blow now and every `interval` after.
            if kind.action == 0 || kind.damage == 0 {
                continue;
            }
            let due = self.enemies[&id].next_attack.is_some_and(|at| now >= at);
            if due && self.barrier_hp > 0 {
                self.enemies.get_mut(&id).unwrap().next_attack = Some(now + seconds(kind.interval));
                let Some(piece) = self.nearest_piece(instance, id) else {
                    continue;
                };
                let mut network = network.lock();
                blow(
                    &mut network,
                    instance,
                    id,
                    kind.action,
                    piece,
                    kind.damage,
                    0,
                );
                self.hurt_barrier(&mut network, instance, kind.damage);
            }
        }
    }

    /// The barrier's piece nearest an enemy.
    fn nearest_piece(&self, instance: &Instance, enemy: ObjectId) -> Option<ObjectId> {
        let from = instance.find_actor(enemy)?.position().0;
        self.barrier
            .iter()
            .filter_map(|piece| {
                let at = instance.find_actor(*piece)?.position().0;
                Some((*piece, (at - from).length_squared()))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(piece, _)| piece)
    }

    /// An enemy's words, to every player.
    fn send_words(&self, network: &mut NetworkState, instance: &Instance, id: ObjectId) {
        let Some(enemy) = self.enemies.get(&id) else {
            return;
        };
        let Some((word, next)) = enemy.words else {
            return;
        };
        let group = waves()
            .enemies
            .get(&enemy.base)
            .map(|kind| kind.group)
            .unwrap_or(0);
        // The barrier's words are each player's own.
        if enemy.full != 0 {
            for (player, (word, next)) in &enemy.own_words {
                command(
                    network,
                    self.director,
                    *player,
                    WORDS,
                    &[id.0, group, WORD_LIST + group, *word, *next, WORDS_LAST],
                );
            }
            return;
        }
        command_all(
            network,
            instance,
            self.director,
            WORDS,
            &[id.0, group, WORD_LIST + group, word, next, WORDS_LAST],
        );
    }

    /// The players' gauges (one starts to drain, one that went round is set full again), and Impassioned Inputs'
    /// end.
    fn gauges(&mut self, network: &mut NetworkState, instance: &Instance, now: Instant) {
        let fever = self.fever_until.is_some();
        for (id, player) in self.players.iter_mut() {
            if fever {
                continue;
            }
            if player.drain_at.is_some_and(|at| now >= at) {
                player.drain_at = None;
                send_gauge(network, *id, player.bonus.gauge, false);
            }
            if player.refill_at.is_some_and(|at| now >= at) {
                player.refill_at = None;
                player.bonus.refill();
                // The drain counts from here.
                player.last_hit = Some(now - seconds(GAUGE_HOLD));
                send_gauge(network, *id, GAUGE_FULL, false);
                send_gauge(network, *id, GAUGE_FULL, false);
            }
        }
        if self.fever_until.is_some_and(|until| now >= until) {
            self.end_fever(network, instance, now);
        }
    }

    /// Impassioned Inputs begins: everyone's gauge is held as it stands.
    fn begin_fever(&mut self, network: &mut NetworkState, instance: &Instance, now: Instant) {
        if self.fever_until.is_none() {
            for (id, player) in self.players.iter_mut() {
                let idle = player
                    .last_hit
                    .map(|at| elapsed(at, now))
                    .unwrap_or(f32::MAX);
                player.bonus.gauge = player.bonus.drained(idle);
                player.drain_at = None;
                send_gauge(network, *id, player.bonus.gauge, true);
            }
            command_all(
                network,
                instance,
                self.director,
                STATE_NOTICE,
                &[FEVER_NOTICE[0], FEVER_STATUS],
            );
            command_all(
                network,
                instance,
                self.director,
                STATE_SWITCH,
                &[FEVER_SWITCH, 1],
            );
        }
        for player in players_in(instance) {
            self.statuses.push(StatusChange::Gain(
                player,
                FEVER_STATUS as u16,
                FEVER_STATUS_PARAM,
                FEVER_SECONDS,
            ));
        }
        self.fever_until = Some(now + seconds(FEVER_SECONDS));
    }

    /// Impassioned Inputs ends: the gauges drain again from here.
    fn end_fever(&mut self, network: &mut NetworkState, instance: &Instance, now: Instant) {
        if self.fever_until.take().is_none() {
            return;
        }
        command_all(
            network,
            instance,
            self.director,
            STATE_NOTICE,
            &[FEVER_NOTICE[1], FEVER_STATUS],
        );
        command_all(
            network,
            instance,
            self.director,
            STATE_SWITCH,
            &[FEVER_SWITCH, 0],
        );
        for (id, player) in self.players.iter_mut() {
            player.last_hit = Some(now - seconds(GAUGE_HOLD));
            send_gauge(network, *id, player.bonus.gauge, false);
        }
    }

    /// What follows an enemy's fall: its words gone for everyone, ActorControl 607, its DeleteActor.
    fn fallen(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        now: Instant,
    ) {
        let mut gone = Vec::new();
        let dead: Vec<ObjectId> = self
            .enemies
            .iter_mut()
            .filter(|(_, enemy)| enemy.dying.is_some_and(|at| now >= at))
            .map(|(id, enemy)| {
                enemy.dying = None;
                *id
            })
            .collect();
        for id in dead {
            kill_actor(network.clone(), instance, id);
        }
        for (id, enemy) in self.enemies.iter_mut() {
            let Some(at) = enemy.fallen else {
                continue;
            };
            if now < at {
                continue;
            }
            let since = elapsed(at, now);
            if enemy.after_fall == 0 && since >= WORDS_GONE_AFTER {
                enemy.after_fall = 1;
                let mut network = network.lock();
                command_all(&mut network, instance, self.director, WORDS_GONE, &[id.0]);
                control(&mut network, instance, *id, CONTROL_WORDS, [0; 4]);
            }
            if enemy.after_fall == 1 && since >= FALLEN_CONTROL_AFTER {
                enemy.after_fall = 2;
                control(
                    &mut network.lock(),
                    instance,
                    *id,
                    CONTROL_FALLEN,
                    [id.0, 1, 0, 100],
                );
            }
            if since >= DELETE_AFTER {
                gone.push(*id);
            }
        }
        for id in gone {
            self.enemies.remove(&id);
            instance.held_npcs.remove(&id);
            network.lock().remove_actor(instance, id);
            if self.barrier_words == Some(id) {
                self.barrier_words = None;
            }
        }
    }

    /// A wave over.
    fn end_wave(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        now: Instant,
    ) {
        let last = self.wave + 1 >= waves().waves.len();
        {
            let mut network = network.lock();
            self.end_fever(&mut network, instance, now);
            self.states_off(&mut network, instance);
            command_all(
                &mut network,
                instance,
                self.director,
                PHASE,
                &[PHASE_BETWEEN],
            );
            if !last {
                command_all(
                    &mut network,
                    instance,
                    self.director,
                    WAVE_CLEARED,
                    &[self.wave as u32 + 1],
                );
            }
        }
        self.wave_bonus(&mut network.lock());
        if last {
            self.finish(network, instance, true, now);
            return;
        }
        {
            let mut network = network.lock();
            command_all(&mut network, instance, self.director, WAVE_OVER, &[]);
            if let Some(image) = IMAGE_WAVE_OVER.get(self.wave) {
                banner(&mut network, instance, *image);
            }
        }
        self.clear_field(network, instance);
        self.clock = now;
        self.cutscene = 0;
        self.stage = Stage::Between { sent: 0 };
    }

    /// Each player's bonus for the barrier's HP (retail: the score as it stood, then with the bonus.)
    fn wave_bonus(&mut self, network: &mut NetworkState) {
        for (id, player) in self.players.iter_mut() {
            command(network, self.director, *id, SCORE, &[player.bonus.score]);
            player.bonus.wave_over(self.barrier_hp);
            command(network, self.director, *id, SCORE, &[player.bonus.score]);
            player.drain_at = None;
            player.refill_at = None;
            player.last_hit = None;
            send_gauge(network, *id, 0.0, false);
        }
    }

    /// The enemies still on the field go (retail deleted them within a second of the wave's end.)
    fn clear_field(&mut self, network: &Arc<Mutex<NetworkState>>, instance: &mut Instance) {
        let ids: Vec<ObjectId> = self.enemies.keys().copied().collect();
        for id in ids {
            self.enemies.remove(&id);
            instance.held_npcs.remove(&id);
            network.lock().remove_actor(instance, id);
        }
        self.barrier_words = None;
        self.pending.clear();
        self.warned = None;
    }

    /// The result's packet for each player: what they won (nothing shown for a lost game, as retail's) and the
    /// four places.
    fn send_result(&self, network: &mut NetworkState, instance: &Instance, won: bool) {
        let entries: Vec<(u64, u64, u32)> = self
            .players
            .iter()
            .filter_map(|(id, player)| match instance.find_actor(*id) {
                Some(NetworkedActor::Player { spawn, .. }) => {
                    Some((spawn.content_id, spawn.account_id, player.bonus.score))
                }
                _ => None,
            })
            .collect();
        let (places, scores, content_ids, account_ids) = result_places(entries);
        for (id, player) in &self.players {
            let (mgp, mgc) = rewards(player.bonus.score);
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::KeyboundResult {
                unk1: won as u32,
                currency_item: ITEM_MGC,
                currency: if won { mgc } else { 0 },
                coffer_item: ITEM_COFFER,
                coffers: won as u32,
                mgp: if won { mgp } else { 0 },
                places,
                scores,
                content_ids,
                account_ids,
            });
            network.send_to_by_actor_id(
                *id,
                FromServer::PacketSegment(ipc, *id),
                DestinationNetwork::ZoneClients,
            );
        }
    }

    /// Each player's rewards: the director's lines, then the MGP, MGC and coffer themselves (their connection's,
    /// `FromServer::KeyboundResult`.)
    fn send_rewards(&self, network: &mut NetworkState, won: bool) {
        for (id, player) in &self.players {
            let (mgp, mgc) = rewards(player.bonus.score);
            let mut logs = vec![(LOG_MGP, vec![mgp, 0]), (LOG_MGC, vec![ITEM_MGC, mgc])];
            if won {
                logs.push((LOG_COFFER, vec![ITEM_COFFER, 1]));
            }
            for (message_type, params) in logs {
                let ipc = EventLogMessage {
                    handler_id: self.director,
                    message_type,
                    params,
                }
                .package()
                .unwrap();
                network.send_to_by_actor_id(
                    *id,
                    FromServer::PacketSegment(ipc, *id),
                    DestinationNetwork::ZoneClients,
                );
            }
            network.send_to_by_actor_id(
                *id,
                FromServer::KeyboundResult {
                    score: player.bonus.score,
                    won,
                },
                DestinationNetwork::ZoneClients,
            );
        }
    }

    /// The game over, as retail's ended. **Won** (the last boss's fall): phase 4, the rewards' lines, the result's
    /// packet and its scene (3, `[4025, 1, 20302]`), `WAVE_OVER` and three `BETWEEN`s; 8.05 s later the result's
    /// numbers and the duty's completion; a player closing the result gets the way out (scene 11) and leaves as that
    /// ends. **Lost** (the barrier at 0): phase 5, the result's packet with nothing won, `WAVE_OVER`, the gauge held
    /// at 0 and the music stopped; 3.05 s later the lost game's music, `LOST` and `[player, 718, 3]` for each; at
    /// 6.1 s the duty's end, the MGP and MGC and the way out.
    fn finish(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        won: bool,
        now: Instant,
    ) {
        {
            let mut network = network.lock();
            self.end_fever(&mut network, instance, now);
            self.states_off(&mut network, instance);
            if won {
                command_all(&mut network, instance, self.director, PHASE, &[PHASE_OVER]);
                self.send_rewards(&mut network, true);
                self.send_result(&mut network, instance, true);
                for player in players_in(instance) {
                    play_scene(
                        &mut network,
                        self.director,
                        player,
                        SCENE,
                        RESULT_FLAGS,
                        &RESULT_PARAMS,
                        SCENE_ARG,
                    );
                }
                command_all(&mut network, instance, self.director, WAVE_OVER, &[]);
                for _ in 0..3 {
                    command_all(&mut network, instance, self.director, BETWEEN, &[]);
                }
            } else {
                command_all(&mut network, instance, self.director, PHASE, &[PHASE_LOST]);
                self.send_result(&mut network, instance, false);
                command_all(&mut network, instance, self.director, WAVE_OVER, &[]);
                for id in self.players.keys() {
                    send_gauge(&mut network, *id, 0.0, true);
                }
                command_all(&mut network, instance, self.director, SET_BGM, &[BGM_NONE]);
            }
        }
        self.clear_field(network, instance);
        self.clock = now;
        self.stage = Stage::Ending { won, step: 0 };
    }

    /// What follows the game's end (`finish`.)
    fn ending(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        won: bool,
        step: u8,
        clock: f32,
    ) {
        if won {
            if step == 0 && clock >= RESULT_AT {
                command_all(
                    &mut network.lock(),
                    instance,
                    self.director,
                    RESULT,
                    &RESULT_ARGS,
                );
                // The duty's completion (DutyCompleted, its time flags cleared, the clear counted.)
                if let Some(director) = instance.directors.first_mut() {
                    director.tasks.push(LuaDirectorTask::CompleteDuty);
                }
                self.stage = Stage::Ending { won, step: 1 };
            }
            return;
        }
        let mut network = network.lock();
        if step == 0 && clock >= LOST_AT[0] {
            command_all(&mut network, instance, self.director, SET_BGM, &[BGM_LOST]);
            command_all(&mut network, instance, self.director, LOST, &[]);
            banner(&mut network, instance, IMAGE_LOST);
            for player in players_in(instance) {
                command_all(
                    &mut network,
                    instance,
                    self.director,
                    PLAYER,
                    &[player.0, PLAYER_LOST[0], PLAYER_LOST[1]],
                );
            }
            self.stage = Stage::Ending { won, step: 1 };
        } else if step == 1 && clock >= LOST_AT[1] {
            command_all(&mut network, instance, self.director, DUTY_COMPLETED, &[]);
            command_all(&mut network, instance, self.director, DUTY_FLAG, &[0]);
            command_all(&mut network, instance, self.director, DUTY_FLAG, &[1]);
            self.send_rewards(&mut network, false);
            for player in players_in(instance) {
                play_scene(
                    &mut network,
                    self.director,
                    player,
                    LEAVE_SCENE,
                    LEAVE_FLAGS,
                    &[],
                    LEAVE_ARG,
                );
            }
            self.stage = Stage::Ending { won, step: 2 };
        }
    }

    /// A player finished one of the game's scenes: the result's is followed by the way out, and that by leaving. The
    /// cutscene before the last wave is followed by nothing.
    fn scene_over(&self, network: &mut NetworkState, player: ObjectId, scene: u16) {
        if !matches!(self.stage, Stage::Ending { .. }) {
            return;
        }
        if scene == SCENE {
            play_scene(
                network,
                self.director,
                player,
                LEAVE_SCENE,
                LEAVE_FLAGS,
                &[],
                LEAVE_ARG,
            );
        } else if scene == LEAVE_SCENE {
            network.send_to_by_actor_id(
                player,
                FromServer::LeaveContent,
                DestinationNetwork::ZoneClients,
            );
        }
    }

    /// A player's word: a hit on `target`, or (the miss) nothing.
    fn word(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        from: ObjectId,
        request: &ActionRequest,
        now: Instant,
    ) {
        let target = request.target.object_id;
        if request.action_id == ACTION_MISS {
            // Retail answered with the action on the player, and nothing else.
            blow(
                &mut network.lock(),
                instance,
                from,
                ACTION_MISS,
                from,
                0,
                request.sequence,
            );
            return;
        }
        // The barrier's words take their own action, and the enemies theirs.
        let charge = ACTION_CHARGES.contains(&request.action_id);
        let live = self.enemies.get(&target).is_some_and(|enemy| {
            enemy.fallen.is_none() && enemy.words.is_some() && (enemy.full != 0) == charge
        });
        if !live || !matches!(self.stage, Stage::Wave { on: true }) {
            // Someone else's word got there first: the blow lands for nothing (retail's effect of kind 1.)
            blow(
                &mut network.lock(),
                instance,
                from,
                request.action_id,
                target,
                0,
                request.sequence,
            );
            return;
        }
        let group = waves()
            .enemies
            .get(&self.enemies[&target].base)
            .map(|kind| kind.group)
            .unwrap_or(0);

        if charge {
            self.charge(network, instance, from, target, request, group, now);
            self.give_statuses(network, instance);
            return;
        }

        // The blow, and the enemy's HP.
        let hp = {
            let enemy = self.enemies.get_mut(&target).unwrap();
            enemy.hp = enemy.hp.saturating_sub(HIT_DAMAGE);
            enemy.hp
        };
        if let Some(actor) = instance.find_actor_mut(target) {
            actor.get_common_spawn_mut().health_points = hp;
        }
        blow(
            &mut network.lock(),
            instance,
            from,
            request.action_id,
            target,
            HIT_DAMAGE,
            request.sequence,
        );
        let death = mechanics()
            .adds
            .get(&self.enemies[&target].base)
            .map(|add| add.death)
            .unwrap_or(0);
        if hp == 0 {
            // A cast it was in is cut off (ActorControl 15 `[540, 1, the action, 1]`, as any caster's that dies.)
            if let Some(casting) = self.enemies.get_mut(&target).unwrap().casting.take() {
                network.lock().send_ac_in_range_instance(
                    instance,
                    target,
                    ActorControlCategory::CancelCast {
                        unk1: 540,
                        unk2: 1,
                        action_id: casting.spell.action,
                        unk3: 1,
                    },
                );
            }
            self.enemies.get_mut(&target).unwrap().to_cast = None;
            if death != 0 {
                // Its own action shows its fall (ActorControl 2 `[2, 2]` with it), and it dies a second later.
                let mut network = network.lock();
                act(
                    &mut network,
                    instance,
                    target,
                    death,
                    Some(target),
                    own_effect(EFFECT_OWN, death),
                    0,
                );
                control(&mut network, instance, target, 2, [2, 2, 0, 0]);
                self.enemies.get_mut(&target).unwrap().dying = Some(now + seconds(DIES_AFTER));
            } else {
                kill_actor(network.clone(), instance, target);
            }
        } else {
            self.counter(network, instance, from, target, now);
        }

        // The player's score and gauge.
        let fever = self.fever_until.is_some();
        let player = self.players.entry(from).or_insert(Player {
            bonus: Bonus::default(),
            last_hit: None,
            drain_at: None,
            refill_at: None,
        });
        if fever {
            // Impassioned Inputs: ten times the word, and the gauge stays held.
            player.bonus.fever_hit(group);
        } else {
            let idle = player
                .last_hit
                .map(|at| elapsed(at, now))
                .unwrap_or(f32::MAX);
            let (_, round) = player.bonus.hit(group, idle);
            player.last_hit = Some(now);
            player.drain_at = Some(now + seconds(GAUGE_HOLD));
            if round {
                player.refill_at = Some(now + seconds(GAUGE_REFILL_AFTER));
            }
        }
        let (score, gauge) = (player.bonus.score, player.bonus.gauge);
        {
            let mut network = network.lock();
            command(&mut network, self.director, from, SCORE, &[score]);
            send_gauge(&mut network, from, gauge, true);
        }

        if hp == 0 {
            let boss = {
                let enemy = self.enemies.get_mut(&target).unwrap();
                // One that dies a second after its own action is a second later in all that follows.
                enemy.fallen = Some(if death != 0 {
                    now + seconds(DIES_AFTER)
                } else {
                    now
                });
                enemy.words = None;
                enemy.boss
            };
            // The first wave counts every kill; the others their big enemy's.
            if self.wave == 0 || boss {
                self.kills += 1;
            }
            let mut network = network.lock();
            command(&mut network, self.director, from, DEFEATED, &[target.0]);
            command(&mut network, self.director, from, WORDS_GONE, &[target.0]);
            command_all(
                &mut network,
                instance,
                self.director,
                ENEMY_DOWN,
                &[target.0, ENEMY_DOWN_ARG],
            );
            if self.wave == 0 || boss {
                command_all(&mut network, instance, self.director, KILLS, &[self.kills]);
            }
            // A 19592's fall is Impassioned Inputs for everyone.
            if self.enemies[&target].base == FEVER_BASE {
                self.begin_fever(&mut network, instance, now);
            }
        } else {
            // Its next word moves up, and a new one comes behind it.
            let most = waves()
                .enemies
                .get(&self.enemies[&target].base)
                .map(|kind| kind.words)
                .unwrap_or(WORD_COUNT[0]);
            let next = self.enemies[&target]
                .words
                .map(|(_, next)| next)
                .unwrap_or(0);
            let new = self.free_word(group, most, |most| fastrand::u32(..most));
            self.enemies.get_mut(&target).unwrap().words = Some((next, new));
            self.send_words(&mut network.lock(), instance, target);
        }
        self.give_statuses(network, instance);
    }
}

/// A player finished one of the director's scenes in the duty.
pub fn scene_finished(
    network: &Arc<Mutex<NetworkState>>,
    instance: &Instance,
    player: ObjectId,
    scene: u16,
) {
    if let Some(game) = instance.keybound.as_ref() {
        game.scene_over(&mut network.lock(), player, scene);
    }
}

/// A player's director trigger in the duty: the client's as it loads (`loaded`), or one of the game's
/// (`Game::trigger`.)
pub fn player_trigger(
    network: &mut NetworkState,
    instance: &mut Instance,
    player: ObjectId,
    id: u32,
    first: u32,
    value: u32,
) {
    if instance.content_finder_condition_id != DUTY {
        return;
    }
    if (id, first) == TRIGGER_LOADED {
        let director = instance
            .keybound
            .as_ref()
            .map(|game| game.director)
            .or(instance.directors.first().map(|director| director.id));
        if let Some(director) = director {
            loaded(network, instance, director, player);
        }
    } else if let Some(game) = instance.keybound.as_mut() {
        game.trigger(player, id, value, Instant::now());
    }
}

/// A player leaves the duty before its end: their typing is turned off first (`WAVE_OVER`.) Bob left mid-wave on
/// 2026-11-15 and came out with the keyboard still the game's: "I can't use esc, can only type in chat, and can't see
/// anything else of my HUD". Nobody left a retail run early, so what retail sends then isn't known.
pub fn player_leaves(network: &mut NetworkState, instance: &Instance, player: ObjectId) {
    if instance.content_finder_condition_id != DUTY {
        return;
    }
    if let Some(game) = instance.keybound.as_ref() {
        command(network, game.director, player, WAVE_OVER, &[]);
    }
}

/// The server's answer to a client that has loaded: the set-up's packet (`KeyboundSetup`), then `BEGIN [0]`.
fn loaded(network: &mut NetworkState, instance: &Instance, director: HandlerId, player: ObjectId) {
    let players = players_in(instance).len() as u32;
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::KeyboundSetup {
        empty_places: PLACES.saturating_sub(players),
        unk1: SETUP[0],
        first: SETUP[1],
        last: SETUP[2],
        seed: fastrand::u32(..),
        barrier: BARRIER_HP,
        time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_millis() as u64)
            .unwrap_or(0),
    });
    network.send_to_by_actor_id(
        player,
        FromServer::PacketSegment(ipc, player),
        DestinationNetwork::ZoneClients,
    );
    command(network, director, player, BEGIN, &[]);
}

/// A player's ActionRequest in the duty: true when it was a word (and so the game's, not the battle's.)
pub fn player_action(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    from: ObjectId,
    request: &ActionRequest,
) -> bool {
    if instance.content_finder_condition_id != DUTY || !is_word(request.action_id) {
        return false;
    }
    let Some(mut game) = instance.keybound.take() else {
        return false;
    };
    game.word(network, instance, from, request, Instant::now());
    instance.keybound = Some(game);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use kawari::ipc::zone::ServerZoneIpcData;

    #[test]
    fn the_waves_read() {
        let data = waves();
        assert_eq!(data.kills_to_clear_wave_1, 25);
        assert_eq!(data.waves.len(), 3);
        // Retail's three line-ups of the first wave: 29, 33 and 31 enemies, each enough for its 25 kills.
        let sizes: Vec<usize> = data.waves[0].sets.iter().map(Vec::len).collect();
        assert_eq!(sizes, [29, 33, 31]);
        // Every enemy of every line-up is a kind the data has, sets off after its SpawnNpc, and no earlier than
        // the wave's start (4.9 s after its enemies appear.)
        for wave in &data.waves {
            for entry in wave.sets.iter().flatten() {
                assert!(data.enemies.contains_key(&entry.base), "{entry:?}");
                assert!(entry.active > entry.spawn, "{entry:?}");
                assert!(entry.active >= 4.9, "{entry:?}");
            }
            for (base, _, _) in &wave.bosses {
                assert!(data.enemies.contains_key(base));
            }
        }
        assert_eq!(data.waves[1].bosses.len(), 3);
        assert_eq!(data.waves[2].bosses, vec![(19596, 0.0, 0.0)]);
        assert_eq!(data.enemies[&19596].name, 14933);
        assert_eq!(data.enemies[&19596].interval, 4.0);
        assert_eq!(data.enemies[&19592].words, 21);
    }

    #[test]
    fn an_enemy_s_hp_by_the_players() {
        let data = waves();
        // Retail's with four and with three.
        assert_eq!(data.enemies[&19587].hp_for(4), 300);
        assert_eq!(data.enemies[&19587].hp_for(3), 200);
        assert_eq!(data.enemies[&19596].hp_for(4), 8400);
        assert_eq!(data.enemies[&19596].hp_for(3), 6600);
        // Fewer weren't seen: the same again off for each, never under one hit.
        assert_eq!(data.enemies[&19596].hp_for(1), 3000);
        assert_eq!(data.enemies[&19587].hp_for(1), 100);
        assert_eq!(data.enemies[&19586].hp_for(1), 200);
        // Not seen with three: about 0.775 of it, to the hundred (4800: 1,100 a player.)
        assert_eq!(data.enemies[&19593].hp_for(3), 3700);
        assert_eq!(data.enemies[&19593].hp_for(8), 4800);
    }

    #[test]
    fn a_line_up_has_its_wave_s_big_enemy() {
        let data = waves();
        let first = Game::line_up(&data.waves[0], |_| 1);
        assert_eq!(first.len(), 33);
        assert!(first.windows(2).all(|pair| pair[0].spawn <= pair[1].spawn));
        let second = Game::line_up(&data.waves[1], |n| n - 1);
        let boss = second.iter().find(|entry| entry.base == 19595).unwrap();
        assert_eq!((boss.x, boss.z, boss.active), (0.0, -24.0, 78.0));
        let last = Game::line_up(&data.waves[2], |_| 0);
        assert_eq!(last.len(), 1);
        assert_eq!((last[0].base, last[0].active), (19596, 5.0));
    }

    /// Retail's run 150 of Connor's captures: a hundred a word until the gauge had gone round twice, then 120.
    #[test]
    fn the_bonus_gauge_and_its_levels() {
        let mut bonus = Bonus::default();
        // From nothing: 1,500, whatever the wait.
        assert_eq!(bonus.hit(0, f32::MAX), (100, false));
        assert_eq!(bonus.gauge, 1500.0);
        // 0.535 s held, then 620 a second: 1.535 s on, 620 fewer.
        assert!((bonus.drained(1.535) - 880.0).abs() < 0.01);
        assert_eq!(bonus.drained(0.3), 1500.0);
        assert_eq!(bonus.drained(60.0), 0.0);
        // Round the first time: no level, and it's set full a moment later.
        bonus.gauge = 9562.0;
        assert_eq!(bonus.hit(0, 0.0), (100, true));
        assert_eq!((bonus.level, bonus.armed), (0, false));
        assert!((bonus.gauge - 1062.0).abs() < 0.01);
        bonus.refill();
        assert_eq!((bonus.gauge, bonus.armed), (10_000.0, true));
        // Round again: the word still scores at the old level, the next ones a fifth more.
        assert_eq!(bonus.hit(0, 0.5), (100, false));
        assert_eq!((bonus.level, bonus.armed), (1, false));
        assert_eq!(bonus.hit(0, 0.5).0, 120);
        // The big enemies' words: 200 and 300 at the same level, 3,000 and 4,500 on the gauge.
        assert_eq!(bonus.hit(1, 0.0).0, 240);
        assert_eq!(bonus.hit(2, 0.0).0, 360);
        // Five levels at most: twice a word's worth.
        bonus.level = 9;
        bonus.armed = true;
        bonus.gauge = 9999.0;
        bonus.hit(0, 0.0);
        assert_eq!(bonus.level, 5);
        assert_eq!(bonus.hit(0, 0.0).0, 200);
        // A wave over: the barrier's share of 10,000, and the gauge and level begin again (retail's +9175 with
        // the barrier at 36,700.)
        let before = bonus.score;
        assert_eq!(bonus.wave_over(36_700), 9175);
        assert_eq!(bonus.score, before + 9175);
        assert_eq!((bonus.gauge, bonus.level, bonus.armed), (0.0, 0, false));
        assert_eq!(bonus.wave_over(40_000), 10_000);
    }

    #[test]
    fn a_score_s_rewards() {
        // Retail's results, to the unit.
        assert_eq!(rewards(13_800), (634, 80));
        assert_eq!(rewards(27_665), (1272, 160));
        assert_eq!(rewards(44_680), (2055, 259));
        assert_eq!(rewards(59_340), (2729, 344));
        assert_eq!(rewards(59_980), (2759, 347));
        assert_eq!(rewards(60_915), (2800, 350));
        assert_eq!(rewards(71_977), (2800, 350));
        assert_eq!(rewards(0), (0, 0));
    }

    /// An enemy's words as they go out: retail's ActorControlSelf 109 `[director, 0x10000008, enemy, 0, 2000010, 352,
    /// 138, 100]` (Connor's capture 933b9471), forty bytes.
    #[test]
    fn a_command_goes_out_as_retail_s() {
        let command = kawari::ipc::zone::ActorControlSelf {
            category: ActorControlCategory::DirectorEvent {
                handler_id: HandlerId(0x8003_DEA9),
                event: DirectorEvent::Raw {
                    id: WORDS,
                    args: [1073811051, 0, WORD_LIST, 352, 138, WORDS_LAST, 0],
                },
            },
        };
        let mut bytes = std::io::Cursor::new(Vec::new());
        binrw::BinWrite::write_le(&command, &mut bytes).unwrap();
        let bytes = bytes.into_inner();
        assert_eq!(bytes.len(), 40);
        let words: Vec<u32> = bytes
            .chunks(4)
            .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
            .collect();
        assert_eq!(
            words,
            [
                109, 2147737257, 268435464, 1073811051, 0, 2000010, 352, 138, 100, 0
            ]
        );
    }

    /// A word typed at an enemy, through to its fall, as each player's client is told it (retail's order: the blow,
    /// the score, the enemy's next words; at 0 its fall, the words gone for the one whose word did it, the count.)
    #[test]
    fn a_typed_word_hits_and_the_third_fells() {
        use crate::server::test_world::{TestWorld, packet_from};
        let mut world = TestWorld::new();
        let player = world.player(1, [4.5, 0.0, 26.0]);
        let target = world.enemy(2, [0.0, 0.0, -18.0], 300);
        world.instance.content_finder_condition_id = DUTY;
        let director = HandlerId(0x8003_DEA9);
        let now = Instant::now();
        let mut game = Game::new(director, now, 4);
        game.stage = Stage::Wave { on: true };
        game.enemies.insert(
            target,
            Enemy {
                words: Some((20, 186)),
                ..Enemy::new(19587, 300, now)
            },
        );
        world.instance.keybound = Some(game);
        world.sent(player);

        let request = |action_id, target: ObjectId| ActionRequest {
            action_id,
            action_type: ActionType::Action,
            sequence: 7,
            target: ObjectTypeId {
                object_id: target,
                object_type: ObjectTypeKind::None,
            },
            ..Default::default()
        };
        let commands = |messages: &[FromServer]| -> Vec<(u32, Vec<u32>)> {
            messages
                .iter()
                .filter_map(|message| match message {
                    FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                        event: DirectorEvent::Raw { id, args },
                        ..
                    }) => Some((*id, args.to_vec())),
                    _ => None,
                })
                .collect()
        };

        // The first word: 100 off, 100 scored, and its next word moves up.
        assert!(player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_HITS[1], target)
        ));
        let sent = world.sent(player);
        assert!(
            sent.iter()
                .any(|message| packet_from(message, player, |data| {
                    matches!(data, ServerZoneIpcData::ActionEffect1(effect)
                if effect.action_id == ACTION_HITS[1] && effect.source_sequence == 7
                    && matches!(effect.effects[0].0, TargetEffectKind::Damage { amount: 100, .. }))
                }))
        );
        let sent = commands(&sent);
        assert_eq!(sent[0], (SCORE, vec![100, 0, 0, 0, 0, 0, 0]));
        assert_eq!(sent[1].0, WORDS);
        assert_eq!(sent[1].1[..4], [target.0, 0, WORD_LIST, 186]);
        assert_eq!(sent[1].1[5], WORDS_LAST);
        assert_ne!(sent[1].1[4], 186);
        assert_eq!(
            world
                .instance
                .find_actor(target)
                .unwrap()
                .get_common_spawn()
                .health_points,
            200
        );

        // The third fells it: no new words, but its fall and the wave's count.
        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_HITS[0], target),
        );
        world.sent(player);
        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_HITS[1], target),
        );
        let sent = commands(&world.sent(player));
        assert_eq!(
            sent,
            vec![
                (SCORE, vec![300, 0, 0, 0, 0, 0, 0]),
                (DEFEATED, vec![target.0, 0, 0, 0, 0, 0, 0]),
                (WORDS_GONE, vec![target.0, 0, 0, 0, 0, 0, 0]),
                (ENEMY_DOWN, vec![target.0, ENEMY_DOWN_ARG, 0, 0, 0, 0, 0]),
                (KILLS, vec![1, 0, 0, 0, 0, 0, 0]),
            ]
        );
        let game = world.instance.keybound.as_ref().unwrap();
        assert_eq!(game.kills, 1);
        assert!(game.enemies[&target].fallen.is_some());

        // A word at it afterwards lands for nothing, and one nobody carried is only the miss.
        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_HITS[0], target),
        );
        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_MISS, player),
        );
        assert!(commands(&world.sent(player)).is_empty());
        // Any other action is the battle's.
        assert!(!player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(7, target)
        ));
    }

    /// The result's four places, as retail's packet had them (in the order of the content IDs; equal scores share a
    /// place.)
    #[test]
    fn the_result_s_places() {
        let (places, scores, content_ids, account_ids) =
            result_places(vec![(30, 3, 500), (10, 1, 900), (20, 2, 900)]);
        assert_eq!(places, [1, 1, 3, 0]);
        assert_eq!(scores, [900, 900, 500, 0]);
        assert_eq!(content_ids, [10, 20, 30, 0]);
        assert_eq!(account_ids, [1, 2, 3, 0]);
    }

    /// Impassioned Inputs: 1,000 a word (retail's 533 of 548; 3,000 the rest) whatever the level, and the gauge held.
    #[test]
    fn a_word_under_impassioned_inputs() {
        let mut bonus = Bonus {
            gauge: 4_000.0,
            level: 3,
            ..Default::default()
        };
        assert_eq!(bonus.fever_hit(0), 1000);
        assert_eq!(bonus.fever_hit(1), 3000);
        assert_eq!((bonus.score, bonus.gauge, bonus.level), (4000, 4_000.0, 3));
    }

    /// As a 19592 falls: the gauge held where it stood (`KeyboundGauge` with `held` 1), the notice `[17835, 5342]` and
    /// switch `[3078, 1]`, and status 5342 `[1180]` for 20 s on each player; a word then is worth 1,000 and comes as
    /// 51027 or 51028; 20 s later the notice `[17836, 5342]`, the switch off and the gauge draining again.
    #[test]
    fn impassioned_inputs_begins_and_ends() {
        use crate::server::test_world::{TestWorld, packet_from};
        let mut world = TestWorld::new();
        let player = world.player(1, [4.5, 0.0, 26.0]);
        let target = world.enemy(2, [0.0, 0.0, -18.0], 400);
        let other = world.enemy(3, [4.0, 0.0, -18.0], 300);
        world.instance.content_finder_condition_id = DUTY;
        let now = Instant::now();
        let mut game = Game::new(HandlerId(0x8003_DEA9), now, 4);
        game.stage = Stage::Wave { on: true };
        for (id, base, hp) in [(target, FEVER_BASE, 100), (other, 19587, 300)] {
            game.enemies.insert(
                id,
                Enemy {
                    words: Some((1, 2)),
                    ..Enemy::new(base, hp, now)
                },
            );
        }
        world.instance.keybound = Some(game);
        world.sent(player);
        let request = |action_id, target: ObjectId| ActionRequest {
            action_id,
            action_type: ActionType::Action,
            target: ObjectTypeId {
                object_id: target,
                object_type: ObjectTypeKind::None,
            },
            ..Default::default()
        };
        let commands = |messages: &[FromServer]| -> Vec<(u32, [u32; 2])> {
            messages
                .iter()
                .filter_map(|message| match message {
                    FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                        event: DirectorEvent::Raw { id, args },
                        ..
                    }) => Some((*id, [args[0], args[1]])),
                    _ => None,
                })
                .collect()
        };
        let gauges = |messages: &[FromServer]| -> Vec<(u32, u32)> {
            messages
                .iter()
                .filter_map(|message| match message {
                    FromServer::PacketSegment(segment, _) => match &segment.data {
                        ServerZoneIpcData::KeyboundGauge { held, gauge, .. } => {
                            Some((*held, *gauge))
                        }
                        _ => None,
                    },
                    _ => None,
                })
                .collect()
        };

        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_HITS[0], target),
        );
        let sent = world.sent(player);
        // The word's own gauge (1,500 for its 100), then held as it stands.
        assert_eq!(gauges(&sent), [(1, 1500), (1, 1500)]);
        let director = commands(&sent);
        assert!(director.contains(&(STATE_NOTICE, [FEVER_NOTICE[0], FEVER_STATUS])));
        assert!(director.contains(&(STATE_SWITCH, [FEVER_SWITCH, 1])));
        assert!(sent.iter().any(|message| matches!(
            message,
            FromServer::ActorControlSelf(ActorControlCategory::GainEffect {
                effect_id: 5342,
                param: 1180,
                ..
            })
        )));

        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_HITS[2], other),
        );
        let sent = world.sent(player);
        assert!(sent.iter().any(|message| packet_from(message, player, |data| {
            matches!(data, ServerZoneIpcData::ActionEffect1(effect) if effect.action_id == 51027)
        })));
        assert_eq!(commands(&sent)[0], (SCORE, [1100, 0]));
        assert_eq!(gauges(&sent), [(1, 1500)]);

        let mut game = world.instance.keybound.take().unwrap();
        game.gauges(
            &mut world.network.lock(),
            &world.instance,
            now + seconds(FEVER_SECONDS + 0.5),
        );
        let sent = world.sent(player);
        assert_eq!(
            commands(&sent),
            [
                (STATE_NOTICE, [FEVER_NOTICE[1], FEVER_STATUS]),
                (STATE_SWITCH, [FEVER_SWITCH, 0])
            ]
        );
        assert_eq!(gauges(&sent), [(0, 1500)]);
        assert!(game.fever_until.is_none());
    }

    #[test]
    fn a_word_nobody_carries() {
        let mut game = Game::new(HandlerId(0x8003_DEA9), Instant::now(), 1);
        let enemy = |words| Enemy {
            words,
            ..Enemy::new(19587, 300, Instant::now())
        };
        game.enemies.insert(ObjectId(1), enemy(Some((3, 4))));
        game.enemies.insert(ObjectId(2), enemy(Some((5, 6))));
        // The picks land on carried words until one is free.
        let mut picks = [3u32, 6, 4, 7].into_iter();
        assert_eq!(game.free_word(0, 425, |_| picks.next().unwrap()), 7);
        // Another group's words don't count.
        assert_eq!(game.free_word(1, 967, |_| 3), 3);
        // Every pick taken: the lowest free one.
        assert_eq!(game.free_word(0, 425, |_| 3), 0);
    }
}
