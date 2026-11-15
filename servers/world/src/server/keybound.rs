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
//! bonus gauge (15 times the word's worth more; from 0.535 s later it drains) and the enemy's next words. The gauge's
//! own packet (opcode 330 in patch 7.57: whether it's held, its value, the server's time) isn't sent: its opcode
//! in 7.58 isn't known, so the client's gauge doesn't move, but the levels it stands for are counted here. An enemy at 0: ActorControls 14 and 2 `[2]`, `DEFEATED` and `WORDS_GONE` for the one whose word did
//! it, `ENEMY_DOWN [enemy, 3068]`, `KILLS`; 2.0 s later its words go for everyone (`WORDS_GONE`, ActorControl 54
//! `[0]`), at 3.04 s ActorControl 607 `[enemy, 1, 0, 100]`, at 4.32 s it's deleted.
//!
//! **The end**: the barrier at 0 loses it; the last boss's fall wins it. Either way each player gets MGP and MGC by
//! their score (`rewards`) and, for a win, a Keybound Coffer.
//!
//! What isn't built is in `docs/src/fork/known-issues.md`: the big enemies' own attacks and summons, the casters'
//! spells, Impassioned Inputs, the words over the barrier, the cutscene before the last wave and the result window.

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
        DamageType, EventLogMessage, TargetEffect, TargetEffectKind,
    },
};
use parking_lot::Mutex;
use serde_json::Value;

use crate::{
    FromServer, GameData,
    server::{
        actor::{NetworkedActor, create_npc_common_spawn, kill_actor},
        director::LuaDirectorTask,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
    },
};

/// The duty (its ContentFinderCondition.)
pub const DUTY: u16 = 1095;

/// A word that hit (the client alternates the two) and one nobody carried.
pub const ACTION_HITS: [u32; 2] = [49516, 49517];
pub const ACTION_MISS: u32 = 49520;

/// The director's commands (ActorControlSelf 109 `[director, command, …]`), as retail sent them.
const WAVE_ON: u32 = 0x1000_0001;
const WAVE_OVER: u32 = 0x1000_0002;
const BARRIER_SET: u32 = 0x1000_0003;
const BARRIER_NOW: u32 = 0x1000_0004;
const APPEAR: u32 = 0x1000_0006;
const PLAYER: u32 = 0x1000_0007;
const WORDS: u32 = 0x1000_0008;
const BEGIN: u32 = 0x1000_0009;
const SCORE: u32 = 0x1000_000A;
const WORDS_GONE: u32 = 0x1000_000B;
const BETWEEN: u32 = 0x1000_000C;
const KILLS: u32 = 0x1000_000D;
const DEFEATED: u32 = 0x1000_0012;
const PHASE: u32 = 0x1000_0013;
const WAVE_CLEARED: u32 = 0x1000_0014;
const ENEMY_DOWN: u32 = 0x1000_0016;

/// `PHASE`'s values: between waves, the enemies appearing, the wave on, the game over.
const PHASE_BETWEEN: u32 = 0;
const PHASE_APPEAR: u32 = 1;
const PHASE_ON: u32 = 2;
const PHASE_OVER: u32 = 4;

/// `PLAYER`'s numbers: `[player, 12609, 12610]` once before the first wave, `[player, 708]` after each wave.
const PLAYER_AT_START: [u32; 2] = [12609, 12610];
const PLAYER_AFTER_WAVE: u32 = 708;
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

/// A hit's damage, and what a word is worth by its group.
const HIT_DAMAGE: u32 = 100;
const WORD_POINTS: [u32; 3] = [100, 200, 300];
/// The bonus gauge: full at this; a hit adds 15 times the word's worth; it drains by about this a second (586 to 702
/// in the clean stretches) from 0.535 s after a hit. Full, it goes round, and 1.2 s later it's set full again; going
/// round after that is a bonus level (each a fifth more a word, five at most.)
const GAUGE_FULL: f32 = 10_000.0;
const GAUGE_PER_POINT: f32 = 15.0;
const GAUGE_DRAIN: f32 = 620.0;
const GAUGE_HOLD: f32 = 0.535;
const GAUGE_REFILL_AFTER: f32 = 1.2;
const MOST_LEVELS: u32 = 5;
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
    /// When the gauge is set full again after going round.
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
}

#[derive(Debug)]
enum Stage {
    /// Before the first wave: the players' numbers, then the enemies.
    Start {
        players_sent: bool,
    },
    /// A wave's enemies come and the wave runs; `on` once `WAVE_ON` went out.
    Wave {
        on: bool,
    },
    /// After a wave: the two `BETWEEN`s, then the next.
    Between {
        sent: u8,
    },
    Over,
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
    let rotation = instance
        .find_actor(from)
        .map(|actor| actor.rotation())
        .unwrap_or(0.0);
    let target = ObjectTypeId {
        object_id: target,
        object_type: ObjectTypeKind::None,
    };
    let mut effects = [TargetEffect::default(); 8];
    effects[0] = TargetEffect(TargetEffectKind::Damage {
        damage_kind: DamageKind::default(),
        damage_type: DamageType::Unique,
        damage_element: DamageElement::Unaspected,
        bonus_percent: 0,
        unk3: 0,
        unk4: 0,
        amount: damage.min(u16::MAX as u32) as u16,
    });
    let ipc = ActionEffect {
        animation_target_id: target,
        targets: vec![target],
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
        }
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
    npc.common.health_points = hp;
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
        command_all(&mut network.lock(), instance, director, BEGIN, &[]);
        instance.keybound = Some(game);
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
                refill_at: None,
            });
        }
        self.gauges(now);
        self.fallen(network, instance, now);

        match self.stage {
            Stage::Start { players_sent } => {
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
                self.enemies_turn(network, instance, now, step);
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
                let appear = if self.wave + 1 == waves().waves.len() - 1 {
                    LAST_APPEAR
                } else {
                    NEXT_APPEAR
                };
                if clock >= appear - 4.5 {
                    self.begin_wave(self.wave + 1, now + seconds(appear - clock));
                }
            }
            Stage::Over => {}
        }
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
                    fallen: None,
                    after_fall: 0,
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
                let mut network = network.lock();
                self.send_words(&mut network, instance, id);
                control(&mut network, instance, id, CONTROL_WORDS, [1, 0, 0, 0]);
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
                self.barrier_hp = self.barrier_hp.saturating_sub(kind.damage);
                command_all(
                    &mut network,
                    instance,
                    self.director,
                    BARRIER_NOW,
                    &[self.barrier_hp],
                );
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
        command_all(
            network,
            instance,
            self.director,
            WORDS,
            &[id.0, group, WORD_LIST + group, word, next, WORDS_LAST],
        );
    }

    /// The players' gauges: one that went round is set full again a moment later.
    fn gauges(&mut self, now: Instant) {
        for player in self.players.values_mut() {
            if player.refill_at.is_some_and(|at| now >= at) {
                player.refill_at = None;
                player.bonus.refill();
                // The drain counts from here.
                player.last_hit = Some(now - seconds(GAUGE_HOLD));
            }
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
        for (id, enemy) in self.enemies.iter_mut() {
            let Some(at) = enemy.fallen else {
                continue;
            };
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
        command_all(&mut network.lock(), instance, self.director, WAVE_OVER, &[]);
        self.clear_field(network, instance);
        self.clock = now;
        self.stage = Stage::Between { sent: 0 };
    }

    /// Each player's bonus for the barrier's HP (retail: the score as it stood, then with the bonus.)
    fn wave_bonus(&mut self, network: &mut NetworkState) {
        for (id, player) in self.players.iter_mut() {
            command(network, self.director, *id, SCORE, &[player.bonus.score]);
            player.bonus.wave_over(self.barrier_hp);
            command(network, self.director, *id, SCORE, &[player.bonus.score]);
            player.refill_at = None;
            player.last_hit = None;
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
    }

    /// The game over: won as the last boss falls, lost as the barrier breaks. Each player's rewards go to their
    /// connection (`FromServer::KeyboundResult`), and the duty is complete or left.
    fn finish(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        won: bool,
        _now: Instant,
    ) {
        {
            let mut network = network.lock();
            command_all(&mut network, instance, self.director, PHASE, &[PHASE_OVER]);
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
            command_all(&mut network, instance, self.director, WAVE_OVER, &[]);
        }
        self.clear_field(network, instance);
        self.stage = Stage::Over;
        // The duty's own ending: complete (the exit and its window), or failed.
        if let Some(director) = instance.directors.first_mut() {
            if won {
                director.tasks.push(LuaDirectorTask::ShowExits);
                director.tasks.push(LuaDirectorTask::CompleteDuty);
            } else {
                for actor_id in self.players.keys().copied() {
                    director
                        .tasks
                        .push(LuaDirectorTask::AbandonDuty { actor_id });
                }
            }
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
        let live = self
            .enemies
            .get(&target)
            .is_some_and(|enemy| enemy.fallen.is_none() && enemy.words.is_some());
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
        if hp == 0 {
            kill_actor(network.clone(), instance, target);
        }

        // The player's score and gauge.
        let player = self.players.entry(from).or_insert(Player {
            bonus: Bonus::default(),
            last_hit: None,
            refill_at: None,
        });
        let idle = player
            .last_hit
            .map(|at| elapsed(at, now))
            .unwrap_or(f32::MAX);
        let (_, round) = player.bonus.hit(group, idle);
        player.last_hit = Some(now);
        if round {
            player.refill_at = Some(now + seconds(GAUGE_REFILL_AFTER));
        }
        let score = player.bonus.score;
        command(&mut network.lock(), self.director, from, SCORE, &[score]);

        if hp == 0 {
            let boss = {
                let enemy = self.enemies.get_mut(&target).unwrap();
                enemy.fallen = Some(now);
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
    }
}

/// A player's ActionRequest in the duty: true when it was a word (and so the game's, not the battle's.)
pub fn player_action(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    from: ObjectId,
    request: &ActionRequest,
) -> bool {
    if instance.content_finder_condition_id != DUTY
        || !(ACTION_HITS.contains(&request.action_id) || request.action_id == ACTION_MISS)
    {
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

    #[test]
    fn a_word_nobody_carries() {
        let mut game = Game::new(HandlerId(0x8003_DEA9), Instant::now(), 1);
        let enemy = |words| Enemy {
            base: 19587,
            hp: 300,
            words,
            active_at: None,
            ready_sent: true,
            arrived: false,
            next_attack: None,
            boss: false,
            fallen: None,
            after_fall: 0,
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
