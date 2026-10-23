//! Party bots: player-like NPCs that join a player, follow the party's tank (or the player) and fight by their class's
//! role. See `docs/src/fork/party-bots.md`.
//!
//! A bot is a battle NPC (BattleNpcSubKind::NpcPartyMember, as retail's Duty Support members are) with a player's
//! look: a random race and face, a made-up name, and the best gear its class can wear at its level, all read from the
//! game data at runtime. Its stats are a player's (the class, race, level and that gear, by the same formulas), and it
//! fights with the actions its class has learned by its level that Kawari has scripts for, through the NPC class
//! rotations (`server/rotation.rs`).
//!
//! Every server tick, before the NPCs move (`npc_behavior`), each bot is told what to do:
//! * out of combat it follows the party's tank (a tank bot, or the player if they're a tank), or the player;
//! * once anything is fighting the party (an NPC with any of them on its enmity list), it fights by role: the tank goes
//!   for whatever is attacking someone else and pulls it off them every few seconds; damage dealers attack what the tank
//!   (or the player) is attacking; the healer heals whoever is lowest when someone's below `HEAL_BELOW_PERCENT`, and
//!   otherwise attacks too.
//!
//! Stage 2: entering a level-synced duty alone through the Duty Finder fills the party's other roles with bots at the
//! duty's level, listed in the party list as retail lists a Duty Finder party (`fill_duty_party`, `party_list`).
//!
//! What isn't done yet: bots don't follow you to another zone, don't avoid area attacks, use only single-target
//! actions, and don't know any dungeon's route or rules (stage 3).

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use glam::Vec3A;
use kawari::{
    common::{
        JumpState, MoveAnimationState, MoveAnimationType, ObjectId, ObjectTypeId, Position,
        Rotation, RotationAction,
    },
    config::get_config,
    ipc::zone::{
        BattleNpcSubKind, CharacterDataFlag, CommonSpawn, DisplayFlag, ObjectKind,
        PartyMemberEntry, SpawnNpc,
    },
};
use parking_lot::Mutex;
use physis::{
    race::{Gender, Race, Tribe},
    savedata::chardat::CustomizeData,
};

use crate::{
    FromServer, GameData, ItemInfoQuery,
    gamedata::PartyRoles,
    inventory::{Inventory, Item},
    server::{
        WorldServer,
        actor::{NetworkedActor, NpcState, NpcTarget},
        glamour,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
        npc_behavior::{BEHAVIOR_TICK_SECONDS, NPC_RUN_SPEED, npc_use_action},
        placement::Placement,
    },
    zone_connection::BaseParameters,
};

/// The level bots are made at: the game's highest, so they're useful anywhere once level synced.
pub const BOT_LEVEL: u8 = 100;

/// How far behind the one they follow bots keep, in yalms, out of combat.
const FOLLOW_DISTANCE: f32 = 2.5;
/// How close melee bots get to their target (as an enemy does with its attack range.)
const MELEE_RANGE: f32 = 3.0;
/// How far ranged bots and healers stand from their target.
const RANGED_RANGE: f32 = 15.0;
/// Ranged bots and healers stay within this of the one they follow while fighting.
const RANGED_STAY_WITHIN: f32 = 20.0;

/// The healer heals someone below this much health (in %), the lowest first.
const HEAL_BELOW_PERCENT: f32 = 60.0;
/// How long a player tank took to answer an enemy hitting someone else in the party (their next action hitting it), in
/// seconds: the three other players' tank runs in Bob's Chronofoil capture of five Sastasha runs (2026-10-22), 54 times
/// (median 2.9). The tank bot answers each loose enemy after one of these.
const TANK_REACTIONS: [f32; 54] = [
    0.1, 0.1, 0.4, 0.4, 0.8, 1.4, 1.5, 1.5, 1.5, 1.7, 1.9, 1.9, 1.9, 1.9, 2.0, 2.1, 2.1, 2.1, 2.1,
    2.1, 2.2, 2.2, 2.5, 2.7, 2.8, 2.8, 2.8, 3.0, 3.2, 3.5, 4.5, 4.5, 4.8, 5.0, 5.1, 5.2, 5.6, 5.8,
    5.8, 5.8, 6.5, 7.5, 8.9, 8.9, 9.2, 9.2, 9.6, 9.6, 12.1, 12.2, 13.0, 13.3, 13.4, 15.4,
];
/// The tank uses its area enmity combo (Total Eclipse, Overpower) when this many enemies fighting the party are within
/// its reach: the same runs' tanks made it their main attack whenever two or more were close (Total Eclipse 74 times
/// to 38 single-target hits; Overpower 62 to 43 and 117 to 37), and rarely with one.
const TANK_AOE_ENEMIES: usize = 2;
/// How far the tank's area attacks reach, in yalms (Total Eclipse's and Overpower's EffectRange, 5, and a little for
/// the enemies' size.)
const TANK_AOE_REACH: f32 = 5.5;
/// The tank opens on an enemy further than this with its ranged attack (Shield Lob, Tomahawk; their Range is 20): the
/// same runs' tanks threw it from 6-21 yalms (14 the median) at enemies just joining the fight.
const TANK_PULL_FROM: f32 = 6.0;
const TANK_PULL_RANGE: f32 = 20.0;
/// The tank keeps up with the player while fighting: further than this from them (they've run on ahead, pulling as they
/// go, wall to wall), it goes after them, still pulling with its ranged attack and area combo on the way.
const TANK_KEEP_UP: f32 = 15.0;
/// How far beyond its target the tank stands, across from the party (`Placement::opposite`): the same runs' tanks
/// stood 2.1 yalms from the middle of what they hit (the median.)
const TANK_SPOT: f32 = 2.5;
/// How far ahead of an enemy's list the tank's area attack keeps it, its tank stance's share of enmity.
const TANK_STANCE_ENMITY: u32 = 1_000;
/// How much enmity a pull gives the tank on that enemy (more each time it's needed, as the top of its list grows.)
const PROVOKE_ENMITY: u32 = 50_000;
/// The pace of their weaponskills and spells, in seconds: a player's global cooldown.
const BOT_PACE: f32 = 2.5;
/// A damage over time is re-applied this many seconds before it would wear off.
const DOT_REFRESH_EARLY: f32 = 2.0;

/// NPC auto-attack actions (EnemyAutoAttack_00870 and _00872): melee, and physical ranged.
const AUTO_ATTACK_MELEE: u32 = 870;
const AUTO_ATTACK_RANGED: u32 = 872;

/// ClassJob's Role column.
const ROLE_TANK: u8 = 1;
const ROLE_MELEE: u8 = 2;
const ROLE_RANGED: u8 = 3;
const ROLE_HEALER: u8 = 4;
/// BaseParam IDs of the main stats (ClassJob's PrimaryStat.)
const PRIMARY_DEXTERITY: u8 = 2;

/// Item EquipSlotCategory rows bots pick gear for, and the equipped slot each fills.
const MAIN_HAND: u8 = 1;
const TWO_HANDED: u8 = 13;
const OFF_HAND: u8 = 2;
const ARMOUR_SLOTS: [u8; 9] = [3, 4, 5, 7, 8, 9, 10, 11, 12];
const RING: u8 = 12;

/// What part a bot plays in the party.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotRole {
    Tank,
    Dps,
    Healer,
}

impl BotRole {
    pub fn from_classjob_role(role: u8) -> Self {
        match role {
            ROLE_TANK => Self::Tank,
            ROLE_HEALER => Self::Healer,
            _ => Self::Dps,
        }
    }
}

/// A bot's side of things, kept by its instance (`Instance::party_bots`, by actor ID.)
#[derive(Debug, Clone)]
pub struct PartyBot {
    /// The player it joined.
    pub leader: ObjectId,
    pub role: BotRole,
    /// How close it gets to what it attacks.
    pub attack_range: f32,
    /// Its single-target heal (healers), if it has one.
    pub heal_action: Option<u32>,
    pub next_heal: Instant,
    /// Tanks: their area enmity combo and ranged pull, and when the next of these (on the GCD) may go.
    pub aoe_action: Option<u32>,
    pub pull_action: Option<u32>,
    pub next_special: Instant,
    /// Tanks: enemies after someone else, and when the tank answers each (`TANK_REACTIONS`.)
    pub answers: HashMap<ObjectId, Instant>,
    /// Tanks: enemies it has already hit, so it pulls only fresh ones from range.
    pub pulled: HashSet<ObjectId>,
    /// When it joined, among its party's bots: its place in the formation.
    pub order: usize,
    /// Out of combat it walks itself, along this path to its formation spot (`npc_behavior` leaves it alone then.)
    pub walking_itself: bool,
    pub path: std::collections::VecDeque<Vec3A>,
    /// When it last re-aimed its path at its moving spot.
    pub reaimed: Instant,
}

/// Where the n-th bot following someone stands out of combat, relative to them: (degrees from straight behind, yalms
/// away). Two behind on either side, then one straight behind further back, then out to the sides, so none stand on
/// each other (or on the one they follow.)
const FORMATION: [(f32, f32); 7] = [
    (-35.0, 2.5),
    (35.0, 2.5),
    (0.0, 3.8),
    (-75.0, 3.2),
    (75.0, 3.2),
    (-25.0, 5.0),
    (25.0, 5.0),
];
/// How often a walking bot re-aims its path at its moving formation spot.
const REAIM_EVERY: Duration = Duration::from_millis(1000);

/// A path from `from` to `to` on the navmesh (straight there where it has none: bots walk on regardless.)
fn instance_navmesh_path(navmesh: &crate::navmesh::Navmesh, from: Vec3A, to: Vec3A) -> Vec<Vec3A> {
    if !navmesh.is_available() {
        return vec![to];
    }
    navmesh.calculate_path(from, to, f32::INFINITY)
}

/// A bot this close to its formation spot is there.
const AT_SPOT: f32 = 0.6;
/// The spot moved this far from where it was walking: it sets off for the new one.
const SPOT_MOVED: f32 = 1.0;

/// The formation spot `index` for someone at `position` facing `rotation`.
fn formation_spot(position: Vec3A, rotation: f32, index: usize) -> Vec3A {
    let (degrees, distance) = FORMATION[index % FORMATION.len()];
    // Further rings for parties bigger than the formation.
    let distance = distance + (index / FORMATION.len()) as f32 * 3.5;
    let angle = rotation + std::f32::consts::PI + degrees.to_radians();
    Vec3A::new(
        position.x + distance * angle.sin(),
        position.y,
        position.z + distance * angle.cos(),
    )
}

/// One of a class's actions, as far as a bot's rotation goes.
#[derive(Debug, Clone, Copy)]
struct ClassAction {
    id: u32,
    level: u8,
    gcd: bool,
    combo_from: u32,
    hostile: bool,
    single_target: bool,
    heals: bool,
    /// An area attack around the user (CastType 2.)
    area: bool,
    /// How far it reaches, in yalms (Range; -1 melee.)
    range: i8,
    /// From its script: its potency, and for a damage over time how long it lasts.
    potency: u32,
    dot_seconds: Option<f32>,
}

/// What an action's script says about it, as far as choosing goes: its `POTENCY`, and its `DURATION` if it puts a
/// damage over time on the target (`damage_over_time`.)
pub fn script_facts(source: &str) -> (u32, Option<f32>) {
    let number = |name: &str| {
        source.lines().find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == name)
                .then(|| value.trim().split_whitespace().next()?.parse::<f32>().ok())?
        })
    };
    let potency = number("POTENCY").unwrap_or(0.0) as u32;
    let dot = source
        .contains("damage_over_time")
        .then(|| number("DURATION"))
        .flatten();
    (potency, dot)
}

impl GameData {
    /// The combat class or job called `name`: its name (any language the server reads, ignoring case and spaces), its
    /// English name or its abbreviation, e.g. "whitemage", "White Mage" or "WHM".
    pub fn find_combat_classjob(&mut self, name: &str) -> Option<u8> {
        let wanted = normalise(name);
        self.classjob_sheet
            .into_iter()
            .flatten_subrows()
            .find(|(_, row)| {
                row.Role != 0
                    && [&row.Name, &row.NameEnglish, &row.Abbreviation]
                        .iter()
                        .any(|text| normalise(&text.to_string()) == wanted)
            })
            .map(|(id, _)| id as u8)
    }

    /// A class's Role (1 tank, 2 melee, 3 ranged, 4 healer), ClassJobParent and PrimaryStat.
    pub fn classjob_role(&mut self, classjob: u8) -> Option<(u8, u8, u8)> {
        let row = self.classjob_sheet.row(classjob as u32)?;
        Some((row.Role, row.ClassJobParent, row.PrimaryStat))
    }

    /// The best gear `classjob` can wear at `level`: for each slot, the item of the highest item level it can equip
    /// (rarity, then the newest, breaking ties), PvP gear left out. Returns (EquipSlotCategory, item ID) pairs, a
    /// ring twice.
    pub fn best_gear(&mut self, classjob: u8, level: u8) -> Vec<(u8, u32)> {
        let wears = self.classjob_categories_with(classjob);
        let mut best: HashMap<u8, (u16, u8, u32)> = HashMap::new();
        let mut second_ring: Option<(u16, u8, u32)> = None;
        let rows: Vec<(u32, u8, u8, u8, u16, u8, bool, bool)> = self
            .item_sheet
            .into_iter()
            .flatten_subrows()
            .filter(|(_, row)| row.LevelEquip > 0 && row.EquipSlotCategory > 0)
            .map(|(id, row)| {
                (
                    id,
                    row.EquipSlotCategory,
                    row.ClassJobCategory,
                    row.LevelEquip,
                    row.LevelItem,
                    row.Rarity,
                    row.IsPvP,
                    row.IsUnique,
                )
            })
            .collect();
        for (id, slot, category, level_equip, item_level, rarity, pvp, unique) in rows {
            if pvp || level_equip > level {
                continue;
            }
            let slot = if slot == TWO_HANDED { MAIN_HAND } else { slot };
            if slot != MAIN_HAND && slot != OFF_HAND && !ARMOUR_SLOTS.contains(&slot) {
                continue;
            }
            if !wears.contains(&category) {
                continue;
            }
            let key = (item_level, rarity, id);
            match best.get(&slot) {
                Some(current) if *current >= key => {
                    // Not the best ring, but maybe the best other one.
                    if slot == RING && second_ring.is_none_or(|ring| ring < key) {
                        second_ring = Some(key);
                    }
                }
                current => {
                    if slot == RING {
                        // The old best is now the second ring; a non-unique best can be worn twice.
                        let previous = current.copied();
                        second_ring = if unique { previous } else { Some(key) };
                    }
                    best.insert(slot, key);
                }
            }
        }

        // A two-handed weapon leaves no room for an off-hand item.
        let two_handed = best
            .get(&MAIN_HAND)
            .and_then(|(_, _, id)| self.item_sheet.row(*id))
            .is_some_and(|row| row.EquipSlotCategory == TWO_HANDED);
        if two_handed {
            best.remove(&OFF_HAND);
        }

        let mut gear: Vec<(u8, u32)> = best.iter().map(|(slot, (_, _, id))| (*slot, *id)).collect();
        if let Some((_, _, id)) = second_ring {
            gear.push((RING, id));
        }
        gear.sort();
        gear
    }

    /// Every ClassJobCategory row that includes `classjob` (its column, after the row's name), reading the sheet once.
    fn classjob_categories_with(&mut self, classjob: u8) -> HashSet<u8> {
        let config = get_config();
        let Ok(sheet) = self
            .resource
            .read_excel_sheet_header("ClassJobCategory")
            .and_then(|exh| {
                self.resource
                    .read_excel_sheet(&exh, "ClassJobCategory", config.world.language())
            })
        else {
            return HashSet::new();
        };
        let mut categories = HashSet::new();
        for category in 0..=u8::MAX {
            if sheet
                .row(category as u32)
                .and_then(|row| {
                    row.columns
                        .get(classjob as usize + 1)
                        .and_then(|column| column.into_bool().copied())
                })
                .unwrap_or(false)
            {
                categories.insert(category);
            }
        }
        categories
    }

    /// The actions `classjob` (or its base class, `parent`) has learned by `level` that `usable` says the server can
    /// perform (has a script for): player actions only, no PvP or role actions.
    fn class_actions(
        &mut self,
        classjob: u8,
        parent: u8,
        level: u8,
        usable: &HashMap<u32, String>,
    ) -> Vec<ClassAction> {
        let mut actions = Vec::new();
        for (id, row) in self.action_sheet.into_iter().flatten_subrows() {
            let class = row.ClassJob;
            if !row.IsPlayerAction
                || row.IsPvP
                || row.IsRoleAction
                || row.ClassJobLevel == 0
                || row.ClassJobLevel > level
                || class < 0
                || (class as u8 != classjob && class as u8 != parent)
                || !usable.contains_key(&id)
            {
                continue;
            }
            // The registered path is the script file itself (`KawariLuaState::action_scripts`.)
            let (potency, dot_seconds) = std::fs::read_to_string(&usable[&id])
                .map(|source| script_facts(&source))
                .unwrap_or((0, None));
            actions.push(ClassAction {
                id,
                level: row.ClassJobLevel,
                gcd: row.CooldownGroup == crate::server::GCD_GROUP
                    || row.AdditionalCooldownGroup == crate::server::GCD_GROUP,
                combo_from: row.ActionCombo as u32,
                hostile: row.CanTargetHostile,
                single_target: row.CastType <= 1,
                heals: !row.CanTargetHostile && (row.CanTargetParty || row.CanTargetAlly),
                area: row.CastType == 2,
                range: row.Range,
                potency,
                dot_seconds,
            });
        }
        actions
    }
}

/// Lower case without spaces, apostrophes or hyphens, for matching class names.
fn normalise(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// A bot's rotation from its class's actions (a heuristic until bots' rotations are recorded), single-target attacks
/// only: its abilities off the GCD whenever they're ready; its combos, each step after the one it follows (the highest
/// level first, so a combo runs to its end); its damage over time spells and weaponskills re-applied as they wear off
/// (their scripts' `DURATION`, less a little); and its filler: the highest level combo starter, or else its strongest
/// plain weaponskill or spell (its script's `POTENCY`). Weaker plain ones (Stone once there's Stone II) aren't used.
fn build_rotation(actions: &[ClassAction]) -> Rotation {
    let attacks: Vec<&ClassAction> = actions
        .iter()
        .filter(|action| action.hostile && action.single_target)
        .collect();
    let starters: HashSet<u32> = attacks
        .iter()
        .filter(|action| action.combo_from != 0)
        .map(|action| action.combo_from)
        .collect();
    let by_level_desc = |list: &mut Vec<&ClassAction>| {
        list.sort_by(|a, b| b.level.cmp(&a.level).then(b.id.cmp(&a.id)))
    };

    let mut abilities: Vec<&ClassAction> = attacks.iter().copied().filter(|a| !a.gcd).collect();
    let mut combos: Vec<&ClassAction> = attacks
        .iter()
        .copied()
        .filter(|a| a.gcd && a.combo_from != 0)
        .collect();
    let mut dots: Vec<&ClassAction> = attacks
        .iter()
        .copied()
        .filter(|a| a.gcd && a.combo_from == 0 && a.dot_seconds.is_some())
        .collect();
    let mut plain: Vec<&ClassAction> = attacks
        .iter()
        .copied()
        .filter(|a| a.gcd && a.combo_from == 0 && a.dot_seconds.is_none())
        .collect();
    by_level_desc(&mut abilities);
    by_level_desc(&mut combos);
    by_level_desc(&mut dots);
    by_level_desc(&mut plain);

    let filler = plain
        .iter()
        .find(|a| starters.contains(&a.id))
        .or_else(|| plain.iter().max_by_key(|a| (a.potency, a.level, a.id)))
        .map(|a| a.id);

    let mut entries = Vec::new();
    for action in abilities.into_iter().chain(combos) {
        entries.push(RotationAction {
            action: action.id,
            ..Default::default()
        });
    }
    for action in dots {
        entries.push(RotationAction {
            action: action.id,
            refresh: action
                .dot_seconds
                .map(|seconds| (seconds - DOT_REFRESH_EARLY).max(BOT_PACE)),
            ..Default::default()
        });
    }
    if let Some(filler) = filler {
        entries.push(RotationAction {
            action: filler,
            ..Default::default()
        });
    }
    Rotation {
        pace: BOT_PACE,
        actions: entries,
    }
}

/// A tank's area enmity combo starter: its highest level hostile area weaponskill around itself on the GCD that
/// doesn't follow another (Total Eclipse, Overpower.)
fn tank_aoe_action(actions: &[ClassAction]) -> Option<u32> {
    actions
        .iter()
        .filter(|a| a.hostile && a.area && a.gcd && a.combo_from == 0)
        .max_by_key(|a| (a.level, a.id))
        .map(|a| a.id)
}

/// A tank's ranged attack, for pulling: its single-target weaponskill on the GCD that reaches furthest (Shield Lob,
/// Tomahawk.)
fn tank_pull_action(actions: &[ClassAction]) -> Option<u32> {
    actions
        .iter()
        .filter(|a| a.hostile && a.single_target && a.gcd && a.range as f32 >= TANK_PULL_RANGE)
        .max_by_key(|a| (a.range, a.level, a.id))
        .map(|a| a.id)
}

/// The healer's heal: its highest level single-target heal on the GCD.
fn heal_action(actions: &[ClassAction]) -> Option<u32> {
    actions
        .iter()
        .filter(|a| a.heals && a.gcd && a.single_target)
        .max_by_key(|a| (a.level, a.id))
        .map(|a| a.id)
}

/// Puts a glamour's pieces over the gear's looks: each in the slot its EquipSlotCategory fills (as the gear's
/// glamour, or as the item where the slot is empty), with its dyes, and the slots it covers (a coat over the legs)
/// emptied. Only the models change; the bot's stats are its gear's.
fn wear_glamour(game_data: &mut GameData, wearing: &mut Inventory, glamour: &glamour::Glamour) {
    for piece in &glamour.pieces {
        let Some(row) = game_data
            .equip_slot_category_sheet
            .row(piece.slot_category as u32)
        else {
            continue;
        };
        let equipped = &mut wearing.equipped;
        let slots: [(i8, &mut Item); 10] = [
            (row.Head, &mut equipped.head),
            (row.Body, &mut equipped.body),
            (row.Gloves, &mut equipped.hands),
            (row.Legs, &mut equipped.legs),
            (row.Feet, &mut equipped.feet),
            (row.Ears, &mut equipped.ears),
            (row.Neck, &mut equipped.neck),
            (row.Wrists, &mut equipped.wrists),
            (row.FingerL, &mut equipped.left_ring),
            (row.FingerR, &mut equipped.right_ring),
        ];
        for (fills, item) in slots {
            match fills {
                1 => {
                    if item.quantity == 0 {
                        *item = Item {
                            item_id: piece.item_id,
                            quantity: 1,
                            ..Default::default()
                        };
                    } else {
                        item.glamour_id = piece.item_id;
                    }
                    item.stains = piece.stains;
                }
                -1 => *item = Item::default(),
                _ => {}
            }
        }
    }
}

/// A random player look: one of the first six races (whose faces and hairstyles 1-4 every tribe and sex has), either
/// sex, random features and colours within the character creator's smallest ranges.
fn random_look() -> CustomizeData {
    let race = Race::from_repr(fastrand::u8(1..=6)).unwrap_or_default();
    let tribe = Tribe::from_repr(race as u8 * 2 - fastrand::u8(0..=1)).unwrap_or_default();
    let gender = if fastrand::bool() {
        Gender::Male
    } else {
        Gender::Female
    };
    let eye_colour = fastrand::u8(0..=30);
    CustomizeData {
        race,
        gender,
        age: 1,
        height: fastrand::u8(0..=100),
        tribe,
        face: fastrand::u8(1..=4),
        hair: fastrand::u8(1..=4),
        enable_highlights: false,
        skin_tone: fastrand::u8(0..=20),
        right_eye_color: eye_colour,
        hair_tone: fastrand::u8(0..=40),
        highlights: 0,
        facial_features: 0,
        facial_feature_color: 0,
        eyebrows: fastrand::u8(0..=2),
        left_eye_color: eye_colour,
        eyes: fastrand::u8(0..=4),
        nose: fastrand::u8(0..=4),
        jaw: fastrand::u8(0..=3),
        mouth: fastrand::u8(0..=3),
        lips_tone_fur_pattern: fastrand::u8(0..=10),
        race_feature_size: fastrand::u8(0..=100),
        race_feature_type: fastrand::u8(1..=4),
        bust: fastrand::u8(0..=100),
        face_paint: 0,
        face_paint_color: 0,
        ..Default::default()
    }
}

/// A made-up first and last name, from syllables (not from the game's text.)
fn random_name() -> String {
    const START: [&str; 16] = [
        "Ka", "Ri", "Sel", "Mo", "Thal", "Ae", "Bran", "Lu", "Vey", "Or", "Ny", "Dar", "Il", "Ce",
        "Fen", "Ja",
    ];
    const MIDDLE: [&str; 12] = [
        "ra", "len", "mi", "do", "va", "ri", "no", "the", "ka", "se", "lo", "",
    ];
    const END: [&str; 12] = [
        "n", "th", "ra", "s", "el", "a", "ix", "or", "e", "yn", "is", "ar",
    ];
    let word = |syllables: usize| {
        let mut word = START[fastrand::usize(..START.len())].to_string();
        for _ in 1..syllables {
            word.push_str(MIDDLE[fastrand::usize(..MIDDLE.len())]);
        }
        word.push_str(END[fastrand::usize(..END.len())]);
        word
    };
    // Retail's limits: each part 2-15 letters, the whole name at most 20 with the space.
    let mut name = format!(
        "{} {}",
        word(fastrand::usize(1..=2)),
        word(fastrand::usize(1..=3))
    );
    name.truncate(20);
    name.trim().to_string()
}

/// A player's stats for `classjob` of `tribe` at `level` wearing `items`, as `ZoneConnection::parameters_at` works
/// them out for players.
fn bot_parameters(
    game_data: &mut GameData,
    classjob: u8,
    tribe: u8,
    level: u8,
    items: &[u32],
    weapon: Option<u32>,
) -> Option<BaseParameters> {
    let modifiers = game_data.get_class_job_modifiers(classjob as u32)?;
    let tribe_row = game_data.get_tribe(tribe)?;
    let param_grow = game_data.get_param_grow(level as u32)?;
    let primary_stat = game_data.get_job_primary_stat(classjob as u16).unwrap_or(1);
    let weapon = weapon.and_then(|id| game_data.get_item_info(ItemInfoQuery::ById(id)));

    let mut parameters = BaseParameters::default();
    parameters.use_retail_formula(level as u16, &param_grow, weapon.as_ref());
    parameters.perform_calculations(primary_stat, &tribe_row, &param_grow, &modifiers);
    let stats: Vec<_> = items
        .iter()
        .filter_map(|id| game_data.equipped_item_stats(*id, true, None))
        .collect();
    parameters.add_equipped_items(&stats);
    parameters.calculate_potencies(&param_grow, Some(&modifiers));
    if let Some(weapon) = &weapon {
        parameters.delay = weapon.delay_ms as u32;
    }
    Some(parameters)
}

/// Spawns a bot of the class called `class_name` next to the player `leader`, and has it join them. `usable` is every
/// action the server has a script for (its path.) Returns the bot's name and class abbreviation, or why it couldn't.
pub fn spawn_party_bot(
    data: &mut WorldServer,
    game_data: &mut GameData,
    usable: &HashMap<u32, String>,
    leader: ObjectId,
    class_name: &str,
) -> Result<String, String> {
    // Spawned as retail spawns a chocobo companion (sub-kind 3, owned, the name in the spawn's text and no BNpcName,
    // character data flags 0x1, no display flags): its nameplate shows that name, which a party member's (sub-kind 9,
    // as retail's Trust) doesn't (Bob tried both in game, 2026-10-21). `kind=partymember` spawns the sub-kind 9 way.
    let words: Vec<&str> = class_name.split_whitespace().collect();
    let as_chocobo = !words.contains(&"kind=partymember");
    let class_name = words
        .iter()
        .filter(|word| !word.starts_with("kind="))
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    let class_name = class_name.as_str();
    let classjob = game_data
        .find_combat_classjob(class_name)
        .ok_or_else(|| format!("No combat class or job is called \"{class_name}\"."))?;
    spawn_bot(
        data, game_data, usable, leader, classjob, BOT_LEVEL, as_chocobo,
    )
    .map(|(_, name)| name)
}

/// Spawns a bot of `classjob` at `level` next to the player `leader`, and has it join them. Returns its actor ID and
/// its name and class abbreviation.
fn spawn_bot(
    data: &mut WorldServer,
    game_data: &mut GameData,
    usable: &HashMap<u32, String>,
    leader: ObjectId,
    classjob: u8,
    level: u8,
    as_chocobo: bool,
) -> Result<(ObjectId, String), String> {
    let (role_column, parent, primary_stat) = game_data
        .classjob_role(classjob)
        .ok_or_else(|| format!("No ClassJob row {classjob}."))?;

    // Gear, worn on an inventory so the model helpers work on it.
    let gear = game_data.best_gear(classjob, level);
    let mut wearing = Inventory::default();
    let mut rings = 0;
    for (slot, item_id) in &gear {
        let Some(info) = game_data.get_item_info(ItemInfoQuery::ById(*item_id)) else {
            continue;
        };
        let item = Item::new(&info, 1);
        let equipped = &mut wearing.equipped;
        match *slot {
            MAIN_HAND => equipped.main_hand = item,
            OFF_HAND => equipped.off_hand = item,
            3 => equipped.head = item,
            4 => equipped.body = item,
            5 => equipped.hands = item,
            7 => equipped.legs = item,
            8 => equipped.feet = item,
            9 => equipped.ears = item,
            10 => equipped.neck = item,
            11 => equipped.wrists = item,
            RING if rings == 0 => {
                equipped.right_ring = item;
                rings += 1;
            }
            RING => equipped.left_ring = item,
            _ => {}
        }
    }
    let item_ids: Vec<u32> = gear.iter().map(|(_, id)| *id).collect();
    let weapon = gear
        .iter()
        .find(|(slot, _)| *slot == MAIN_HAND)
        .map(|(_, id)| *id);

    let look = random_look();
    let name = random_name();
    // Its own look: one of the server's glamours for its sex and class, over its gear (`glamour.rs`; none without the
    // list.)
    let glamours = glamour::glamours(game_data);
    let worn_glamour = glamour::pick(&glamours, look.gender, classjob, parent).map(|glamour| {
        wear_glamour(game_data, &mut wearing, glamour);
        glamour.name.clone()
    });
    let parameters = bot_parameters(
        game_data,
        classjob,
        look.tribe as u8,
        level,
        &item_ids,
        weapon,
    )
    .ok_or("Couldn't work out the bot's stats.")?;

    let actions = game_data.class_actions(classjob, parent, level, usable);
    let rotation = build_rotation(&actions);
    let role = BotRole::from_classjob_role(role_column);
    let heal = if role == BotRole::Healer {
        heal_action(&actions)
    } else {
        None
    };
    let (aoe_action, pull_action) = if role == BotRole::Tank {
        (tank_aoe_action(&actions), tank_pull_action(&actions))
    } else {
        (None, None)
    };
    let ranged = matches!(role_column, ROLE_RANGED | ROLE_HEALER);
    let autoattack = match role_column {
        ROLE_TANK | ROLE_MELEE => AUTO_ATTACK_MELEE,
        ROLE_RANGED if primary_stat == PRIMARY_DEXTERITY => AUTO_ATTACK_RANGED,
        _ => 0,
    };
    let attack_range = if ranged { RANGED_RANGE } else { MELEE_RANGE };

    let instance = data
        .find_actor_instance_mut(leader)
        .ok_or("You aren't in a zone.")?;
    let leader_actor = instance.find_actor(leader).ok_or("You aren't in a zone.")?;
    let mut position = leader_actor.position();
    let rotation_facing = leader_actor.rotation();
    // A little to one side of you, a different spot for each bot.
    let angle =
        rotation_facing + std::f32::consts::PI + (instance.party_bots.len() as f32 - 1.5) * 0.6;
    position.0.x += FOLLOW_DISTANCE * angle.sin();
    position.0.z += FOLLOW_DISTANCE * angle.cos();

    let hp = parameters.hp.max(1);
    let spawn = SpawnNpc {
        common: CommonSpawn {
            main_weapon_model: wearing.get_main_weapon_id(game_data),
            sec_weapon_model: wearing.get_sub_weapon_id(game_data),
            models: wearing.legacy_model_ids(game_data),
            second_model_stain_ids: wearing.second_model_stain_ids(),
            max_health_points: hp,
            health_points: hp,
            max_resource_points: 10000,
            resource_points: 10000,
            // As retail spawns its party-member NPCs (Trust, Bob's Chronofoil captures in 822 and 884: 6 of them):
            // owned by the player, display flags 0x40000. Without them the bots had no nameplate (2026-10-21 bundle.)
            object_kind: ObjectKind::BattleNpc(if as_chocobo {
                BattleNpcSubKind::Chocobo
            } else {
                BattleNpcSubKind::NpcPartyMember
            }),
            owner_id: leader,
            display_flags: if as_chocobo {
                DisplayFlag::empty()
            } else {
                DisplayFlag::UNK1
            },
            battalion: 0,
            model_chara: 0,
            level,
            class_job: classjob,
            position: Position(position.0),
            rotation: rotation_facing,
            name: name.clone(),
            look,
            target_id: ObjectTypeId::default(),
            ..Default::default()
        },
        // Retail's chocobo companions have 0x1 here (its Trust members 0x2, which Kawari reads as hostile.)
        character_data_flags: if as_chocobo {
            CharacterDataFlag::from_bits_retain(0x1)
        } else {
            CharacterDataFlag::empty()
        },
        ..Default::default()
    };

    let order = instance
        .party_bots
        .values()
        .filter(|bot| bot.leader == leader)
        .map(|bot| bot.order + 1)
        .max()
        .unwrap_or(0);
    let actor_id = Instance::generate_actor_id();
    let config = get_config();
    instance.insert_npc(actor_id, spawn, &config, game_data);
    if let Some(NetworkedActor::Npc {
        timeline,
        parameters: npc_parameters,
        state,
        ..
    }) = instance.find_actor_mut(actor_id)
    {
        timeline.autoattack_action_id = autoattack;
        timeline.attack_range = FOLLOW_DISTANCE;
        timeline.hitbox_radius = 0.5;
        timeline.rotation = Some(rotation);
        *npc_parameters = parameters;
        *state = NpcState::Follow;
    }
    instance.party_bots.insert(
        actor_id,
        PartyBot {
            leader,
            role,
            attack_range,
            heal_action: heal,
            next_heal: Instant::now(),
            aoe_action,
            pull_action,
            next_special: Instant::now(),
            answers: HashMap::new(),
            pulled: HashSet::new(),
            order,
            walking_itself: true,
            path: Default::default(),
            reaimed: Instant::now(),
        },
    );

    let abbreviation = game_data
        .classjob_sheet
        .row(classjob as u32)
        .map(|row| row.Abbreviation.to_string())
        .unwrap_or_default();
    tracing::info!(
        "Party bot {name} ({abbreviation}, level {level}, {hp} HP) joined {leader}: gear {item_ids:?}, glamour {worn_glamour:?}, rotation {:?}, heal {heal:?}",
        actions.iter().map(|a| a.id).collect::<Vec<_>>()
    );
    Ok((actor_id, format!("{name} ({abbreviation})")))
}

/// The base classes Duty Finder bots are picked from (Gladiator, Pugilist, Marauder, Lancer, Archer, Conjurer,
/// Thaumaturge, Arcanist, Rogue): the classes Kawari has action scripts for, levels 1-50. Each one's role is read from
/// the ClassJob sheet.
const DUTY_BOT_CLASSES: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 26, 29];

/// The classes of the bots that fill a party of `roles` for a player on `player_classjob`: a role the player plays is
/// theirs, the rest are bots, each of a random class of that role. A DPS player takes a melee or ranged place, their
/// own kind first.
pub fn duty_bot_classes(
    game_data: &mut GameData,
    roles: PartyRoles,
    player_classjob: u8,
) -> Vec<u8> {
    let player_role = game_data
        .classjob_role(player_classjob)
        .map(|(role, _, _)| role)
        .unwrap_or_default();
    let [tanks, healers, melee, ranged] = roles_left(roles, player_role);

    let mut by_role: HashMap<u8, Vec<u8>> = HashMap::new();
    for classjob in DUTY_BOT_CLASSES {
        if let Some((role, _, _)) = game_data.classjob_role(classjob) {
            by_role.entry(role).or_default().push(classjob);
        }
    }
    let mut classes = Vec::new();
    for (role, count) in [
        (ROLE_TANK, tanks),
        (ROLE_HEALER, healers),
        (ROLE_MELEE, melee),
        (ROLE_RANGED, ranged),
    ] {
        for _ in 0..count {
            if let Some(choices) = by_role.get(&role)
                && let Some(classjob) = fastrand::choice(choices)
            {
                classes.push(*classjob);
            }
        }
    }
    // The party is never bigger than the duty allows (the player counts too.)
    classes.truncate(roles.members.saturating_sub(1) as usize);
    classes
}

/// How many tanks, healers, melee and ranged DPS a party of `roles` still needs once a player whose ClassJob Role is
/// `player_role` has joined it.
fn roles_left(roles: PartyRoles, player_role: u8) -> [u8; 4] {
    let (mut tanks, mut healers, mut melee, mut ranged) =
        (roles.tanks, roles.healers, roles.melee, roles.ranged);
    match player_role {
        ROLE_TANK => tanks = tanks.saturating_sub(1),
        ROLE_HEALER => healers = healers.saturating_sub(1),
        ROLE_MELEE if melee > 0 => melee -= 1,
        ROLE_RANGED if ranged > 0 => ranged -= 1,
        _ if melee > 0 => melee -= 1,
        _ => ranged = ranged.saturating_sub(1),
    }
    [tanks, healers, melee, ranged]
}

/// Fills the party of a player who came into the duty `content_finder_condition` through the Duty Finder alone: bots
/// for the roles they don't play, level synced to the duty (ClassJobLevelSync). Unsynced duties get none. Returns the
/// bots' actor IDs.
pub fn fill_duty_party(
    data: &mut WorldServer,
    game_data: &mut GameData,
    usable: &HashMap<u32, String>,
    leader: ObjectId,
    content_finder_condition: u16,
) -> Vec<ObjectId> {
    let Some(level) = game_data.find_content_synced_level(content_finder_condition) else {
        return Vec::new();
    };
    let Some(roles) = game_data.content_party_roles(content_finder_condition) else {
        return Vec::new();
    };
    let Some(player_classjob) = data
        .find_actor_instance(leader)
        .and_then(|instance| instance.find_actor(leader))
        .map(|actor| actor.get_common_spawn().class_job)
    else {
        return Vec::new();
    };

    let mut bots = Vec::new();
    for classjob in duty_bot_classes(game_data, roles, player_classjob) {
        match spawn_bot(data, game_data, usable, leader, classjob, level, true) {
            Ok((actor_id, name)) => {
                tracing::info!(
                    "Duty Finder: {name} fills a place in {leader}'s party (level {level})"
                );
                bots.push(actor_id);
            }
            Err(error) => {
                tracing::warn!("Duty Finder: couldn't add a bot of class {classjob}: {error}")
            }
        }
    }
    bots
}

/// The party list for a player and the bots that joined them, as retail lists a Duty Finder party (Bob's Chronofoil
/// capture of five Sastasha runs, 2026-10-22): each member with their actor ID, class, level, HP and MP, the zone,
/// `sync_positions` 1, no entity or parent ID. The player comes first and leads. The bots' account and content IDs
/// are made up from their actor IDs (no one else has them.) None when the player has no bots.
pub fn party_list(instance: &Instance, leader: ObjectId) -> Option<Vec<PartyMemberEntry>> {
    let Some(NetworkedActor::Player { spawn: player, .. }) = instance.find_actor(leader) else {
        return None;
    };
    let mut bots: Vec<(&ObjectId, &PartyBot)> = instance
        .party_bots
        .iter()
        .filter(|(_, bot)| bot.leader == leader)
        .collect();
    if bots.is_empty() {
        return None;
    }
    bots.sort_by_key(|(_, bot)| bot.order);

    let entry = |common: &CommonSpawn, actor_id: ObjectId, account_id: u64, content_id: u64| {
        PartyMemberEntry {
            name: common.name.clone(),
            account_id,
            content_id,
            actor_id,
            health_points: common.health_points,
            max_health_points: common.max_health_points,
            resource_points: common.resource_points,
            max_resource_points: common.max_resource_points,
            home_world_id: player.home_world_id,
            current_zone_id: instance.zone.id,
            sync_positions: 1,
            classjob_id: common.class_job,
            unk2: 1,
            classjob_level: common.level,
            ..Default::default()
        }
    };
    let mut list = vec![entry(
        &player.common,
        leader,
        player.account_id,
        player.content_id,
    )];
    for (actor_id, _) in bots {
        if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor(*actor_id) {
            let made_up = BOT_ID_BASE | actor_id.0 as u64;
            list.push(entry(&spawn.common, *actor_id, made_up, made_up));
        }
    }
    list.truncate(PartyMemberEntry::NUM_ENTRIES);
    Some(list)
}

/// Bots' made-up account and content IDs: their actor ID with this on top.
const BOT_ID_BASE: u64 = 0x00B0_0000_0000_0000;

/// Removes every bot that joined `leader` from their instance. Returns how many.
pub fn dismiss_party_bots(
    data: &mut WorldServer,
    network: &mut NetworkState,
    leader: ObjectId,
) -> usize {
    let Some(instance) = data.find_actor_instance_mut(leader) else {
        return 0;
    };
    let bots: Vec<ObjectId> = instance
        .party_bots
        .iter()
        .filter(|(_, bot)| bot.leader == leader)
        .map(|(id, _)| *id)
        .collect();
    for id in &bots {
        instance.party_bots.remove(id);
        network.remove_actor(instance, *id);
    }
    bots.len()
}

/// What one enemy fighting the party is doing.
struct Engaged {
    id: ObjectId,
    position: Vec3A,
    /// Whom it's after most.
    after: Option<ObjectId>,
}

/// Tells every bot in this instance what to do this tick (see the module's description.)
pub fn update(
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
) {
    if instance.party_bots.is_empty() {
        return;
    }
    let now = Instant::now();

    // Bots whose player left (or that are gone themselves) leave too.
    let gone: Vec<ObjectId> = instance
        .party_bots
        .iter()
        .filter(|(id, bot)| {
            !instance.actors.contains_key(id) || !instance.actors.contains_key(&bot.leader)
        })
        .map(|(id, _)| *id)
        .collect();
    for id in gone {
        instance.party_bots.remove(&id);
        if instance.actors.contains_key(&id) {
            network.lock().remove_actor(instance, id);
        }
    }

    let leaders: HashSet<ObjectId> = instance.party_bots.values().map(|bot| bot.leader).collect();
    for leader in leaders {
        update_party(network, game_data, instance, leader, now);
    }
}

fn health_percent(actor: &NetworkedActor) -> f32 {
    let common = actor.get_common_spawn();
    if common.max_health_points == 0 {
        100.0
    } else {
        common.health_points as f32 * 100.0 / common.max_health_points as f32
    }
}

fn update_party(
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
    leader: ObjectId,
    now: Instant,
) {
    let bots: Vec<(ObjectId, PartyBot)> = instance
        .party_bots
        .iter()
        .filter(|(_, bot)| bot.leader == leader)
        .map(|(id, bot)| (*id, bot.clone()))
        .collect();
    let mut members: Vec<ObjectId> = vec![leader];
    members.extend(bots.iter().map(|(id, _)| *id));

    // Who the party follows: its tank (the player, if they're one; else a tank bot), or the player.
    let leader_classjob = instance
        .find_actor(leader)
        .map(|actor| actor.get_common_spawn().class_job)
        .unwrap_or(0);
    let leader_is_tank = game_data
        .lock()
        .classjob_role(leader_classjob)
        .is_some_and(|(role, _, _)| role == ROLE_TANK);
    let tank_bot = bots
        .iter()
        .find(|(id, bot)| {
            bot.role == BotRole::Tank
                && instance
                    .find_actor(*id)
                    .is_some_and(|actor| actor.get_common_spawn().health_points > 0)
        })
        .map(|(id, _)| *id);
    let tank = if leader_is_tank {
        Some(leader)
    } else {
        tank_bot
    };
    let leader_target = instance
        .find_actor(leader)
        .map(|actor| actor.get_common_spawn().target_id.object_id)
        .filter(|target| target.is_valid());

    // Everything fighting the party: any NPC (not a bot) with one of them on its enmity list.
    let engaged: Vec<Engaged> = instance
        .actors
        .iter()
        .filter(|(id, _)| !instance.party_bots.contains_key(id))
        .filter_map(|(id, actor)| match actor {
            NetworkedActor::Npc { enmity, spawn, .. }
                if spawn.common.health_points > 0
                    && enmity.keys().any(|hated| members.contains(hated)) =>
            {
                let after = enmity
                    .iter()
                    .max_by_key(|(_, amount)| **amount)
                    .map(|(hated, _)| *hated);
                Some(Engaged {
                    id: *id,
                    position: spawn.common.position.0,
                    after,
                })
            }
            _ => None,
        })
        .collect();
    let is_engaged = |id: ObjectId| engaged.iter().any(|enemy| enemy.id == id);

    // Whom the healer helps first: the lowest of the party below the line.
    let hurt = members
        .iter()
        .filter_map(|id| instance.find_actor(*id).map(|actor| (*id, actor)))
        .filter(|(_, actor)| actor.get_common_spawn().health_points > 0)
        .map(|(id, actor)| (id, health_percent(actor)))
        .filter(|(_, percent)| *percent < HEAL_BELOW_PERCENT)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id);

    // Where everyone is and faces, and each bot's place behind the one it follows.
    let where_is: HashMap<ObjectId, (Vec3A, f32)> = members
        .iter()
        .filter_map(|id| {
            instance
                .find_actor(*id)
                .map(|actor| (*id, (actor.position().0, actor.rotation())))
        })
        .collect();
    let follows = |bot_id: ObjectId| {
        if Some(bot_id) == tank {
            leader
        } else {
            tank.unwrap_or(leader)
        }
    };
    let mut spots: HashMap<ObjectId, usize> = HashMap::new();
    {
        let mut by_order: Vec<&(ObjectId, PartyBot)> = bots.iter().collect();
        by_order.sort_by_key(|(_, bot)| bot.order);
        let mut taken: HashMap<ObjectId, usize> = HashMap::new();
        for (bot_id, _) in by_order {
            let count = taken.entry(follows(*bot_id)).or_insert(0);
            spots.insert(*bot_id, *count);
            *count += 1;
        }
    }

    let mut heals = Vec::new();
    let mut provokes = Vec::new();
    let mut specials: Vec<(ObjectId, u32, ObjectId, Vec<ObjectId>)> = Vec::new();
    let mut tank_answers: Vec<(ObjectId, HashMap<ObjectId, Instant>)> = Vec::new();
    let mut repulls: Vec<(ObjectId, ObjectId)> = Vec::new();
    let mut walks = Vec::new();
    let mut fighting = Vec::new();
    let navmesh = &instance.navmesh;
    for (bot_id, bot) in &bots {
        let Some(NetworkedActor::Npc {
            spawn,
            last_position,
            state,
            enmity,
            navmesh_target,
            navmesh_path,
            navmesh_path_lerp,
            timeline,
            placement,
            cast,
            ..
        }) = instance.actors.get_mut(bot_id)
        else {
            continue;
        };
        if spawn.common.health_points == 0 || *state == NpcState::Dead {
            continue;
        }
        let follow = follows(*bot_id);
        let casting = cast.is_some_and(|cast| cast.until > now);

        if engaged.is_empty() {
            // Out of combat: its spot in the formation behind the one it follows.
            enmity.clear();
            if *state != NpcState::Follow {
                *state = NpcState::Follow;
                navmesh_path.clear();
                *navmesh_path_lerp = 0.0;
            }
            spawn.common.target_id = ObjectTypeId::default();
            *placement = Placement::default();
            let Some((anchor, facing)) = where_is.get(&follow).copied() else {
                continue;
            };
            let spot = formation_spot(anchor, facing, spots.get(bot_id).copied().unwrap_or(0));
            let here = spawn.common.position.0;
            let flat = |a: Vec3A, b: Vec3A| Vec3A::new(a.x - b.x, 0.0, a.z - b.z).length();
            // Out of combat the bot walks itself (`walking_itself`: NPCs' own walking leaves it alone), so nothing
            // else steers it: with both, it reached its spot and was pulled back along its old path, to and fro
            // every half second (Bob's 2026-10-21 bundle).
            *navmesh_target = None;
            navmesh_path.clear();
            *navmesh_path_lerp = 0.0;
            *last_position = None;
            let mut path = bot.path.clone();
            if flat(here, spot) <= AT_SPOT && path.is_empty() {
                walks.push((*bot_id, path, None, bot.reaimed));
                continue;
            }
            // Its path, re-aimed at where its spot is now (from where it stands) at most every `REAIM_EVERY`, and
            // only when the spot has moved off the path's end.
            let drifted = path.back().is_none_or(|end| flat(*end, spot) > SPOT_MOVED);
            let due = path.is_empty() || now.duration_since(bot.reaimed) >= REAIM_EVERY;
            let mut reaimed = bot.reaimed;
            if drifted && due {
                path = instance_navmesh_path(navmesh, here, spot).into();
                if path.back().is_none_or(|end| flat(*end, spot) > AT_SPOT) {
                    path.push_back(spot); // straight on where the navmesh stops
                }
                reaimed = now;
            }
            // A tick's run along it.
            let mut at = here;
            let mut left = NPC_RUN_SPEED * BEHAVIOR_TICK_SECONDS;
            while left > 0.0
                && let Some(next) = path.front().copied()
            {
                let distance = at.distance(next);
                if distance <= left {
                    at = next;
                    left -= distance;
                    path.pop_front();
                } else {
                    at += (next - at) * (left / distance);
                    left = 0.0;
                }
            }
            if flat(at, here) >= 0.05 {
                let facing = f32::atan2(at.x - here.x, at.z - here.z);
                spawn.common.position = Position(at);
                spawn.common.rotation = facing;
                walks.push((*bot_id, path, Some((at, facing)), reaimed));
            } else {
                walks.push((*bot_id, path, None, reaimed));
            }
            continue;
        }
        if bot.walking_itself {
            fighting.push(*bot_id);
        }

        // The healer heals first.
        if bot.role == BotRole::Healer
            && let (Some(heal), Some(patient)) = (bot.heal_action, hurt)
            && now >= bot.next_heal
            && !casting
        {
            heals.push((*bot_id, heal, patient));
            continue;
        }

        let current = match navmesh_target {
            Some(NpcTarget::Actor(target)) if *state == NpcState::Hate => Some(*target),
            _ => None,
        };
        let target = match bot.role {
            BotRole::Tank => {
                // Each enemy after someone else is answered after a player tank's reaction time (`TANK_REACTIONS`):
                // the tank goes for it and takes it (`provokes`), the nearest due first; otherwise it keeps on what
                // it's fighting.
                let position = spawn.common.position.0;
                let loose: Vec<&Engaged> = engaged
                    .iter()
                    .filter(|enemy| enemy.after != Some(*bot_id))
                    .collect();
                // An enemy that's turned on someone else is one to pull again, from range if it's far.
                for enemy in &loose {
                    repulls.push((*bot_id, enemy.id));
                }
                let mut answers = bot.answers.clone();
                answers.retain(|enemy, _| loose.iter().any(|loose| loose.id == *enemy));
                for enemy in &loose {
                    answers.entry(enemy.id).or_insert_with(|| {
                        let delay = TANK_REACTIONS[fastrand::usize(..TANK_REACTIONS.len())];
                        now + Duration::from_secs_f32(delay)
                    });
                }
                let due = loose
                    .iter()
                    .filter(|enemy| answers.get(&enemy.id).is_some_and(|at| *at <= now))
                    .min_by(|a, b| {
                        a.position
                            .distance(position)
                            .total_cmp(&b.position.distance(position))
                    })
                    .map(|enemy| enemy.id);
                if let Some(due) = due {
                    provokes.push((*bot_id, due));
                    answers.remove(&due);
                }
                let target = due
                    .or(current.filter(|id| is_engaged(*id)))
                    .or_else(|| engaged.first().map(|enemy| enemy.id));

                // Its GCD: the ranged pull on a fresh enemy out of reach, else the area combo with enough close.
                if let Some(target) = target
                    && now >= bot.next_special
                    && !casting
                {
                    let distance = engaged
                        .iter()
                        .find(|enemy| enemy.id == target)
                        .map(|enemy| enemy.position.distance(position))
                        .unwrap_or(0.0);
                    let close: Vec<ObjectId> = engaged
                        .iter()
                        .filter(|enemy| enemy.position.distance(position) <= TANK_AOE_REACH)
                        .map(|enemy| enemy.id)
                        .collect();
                    if let Some(pull) = bot.pull_action
                        && !bot.pulled.contains(&target)
                        && (TANK_PULL_FROM..=TANK_PULL_RANGE).contains(&distance)
                    {
                        specials.push((*bot_id, pull, target, Vec::new()));
                    } else if let Some(aoe) = bot.aoe_action
                        && close.len() >= TANK_AOE_ENEMIES
                    {
                        specials.push((*bot_id, aoe, target, close));
                    }
                }
                tank_answers.push((*bot_id, answers));
                target
            }
            BotRole::Dps | BotRole::Healer => {
                // What the tank (or the player) is attacking.
                let tank_target =
                    tank.and_then(|tank| tank_target(&engaged, tank, &bots, leader_target));
                tank_target
                    .or(leader_target.filter(|id| is_engaged(*id)))
                    .or(current.filter(|id| is_engaged(*id)))
                    .or_else(|| engaged.first().map(|enemy| enemy.id))
            }
        };
        let Some(target) = target else {
            continue;
        };

        enmity.clear();
        enmity.insert(target, 1);
        timeline.attack_range = bot.attack_range;
        *placement = if bot.role == BotRole::Tank {
            // Across its target from the rest of the party, so cones and cleaves miss them; and with the player when
            // they've run on ahead (unless the player is the one it follows only because they're the tank.)
            let party: Vec<ObjectId> = members.iter().copied().filter(|id| id != bot_id).collect();
            Placement {
                anchors: if leader_is_tank {
                    Vec::new()
                } else {
                    vec![leader]
                },
                within: TANK_KEEP_UP,
                opposite: Some((party, TANK_SPOT)),
                ..Default::default()
            }
        } else if bot.attack_range > MELEE_RANGE {
            Placement {
                anchors: vec![follow],
                within: RANGED_STAY_WITHIN,
                ..Default::default()
            }
        } else {
            Placement::default()
        };
    }

    for (bot_id, path, moved, reaimed) in walks {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.path = path;
            bot.reaimed = reaimed;
            bot.walking_itself = true;
        }
        if let Some((position, rotation)) = moved {
            network.lock().send_in_range_instance(
                bot_id,
                instance,
                FromServer::ActorMove(
                    bot_id,
                    Position(position),
                    rotation,
                    MoveAnimationType::empty(),
                    MoveAnimationState::None,
                    JumpState::NoneOrFalling,
                    Some((NPC_RUN_SPEED * 10.0) as u8),
                ),
                DestinationNetwork::ZoneClients,
            );
        }
    }
    // Fighting: NPCs' own walking (chasing its target) takes over again.
    for bot_id in fighting {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.walking_itself = false;
            bot.path.clear();
        }
    }
    for (bot_id, answers) in tank_answers {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.answers = answers;
        }
    }
    for (bot_id, enemy) in repulls {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.pulled.remove(&enemy);
        }
    }
    for (bot_id, enemy) in provokes {
        if let Some(NetworkedActor::Npc { enmity, .. }) = instance.actors.get_mut(&enemy) {
            let top = enmity.values().copied().max().unwrap_or(0);
            let tank = enmity.entry(bot_id).or_insert(0);
            *tank = (*tank).max(top).saturating_add(PROVOKE_ENMITY);
        }
    }
    // Tanks' pulls and area attacks, on their GCD: the rotation is told, so it doesn't use another at once.
    for (bot_id, action, target, close) in specials {
        npc_use_action(network, game_data, instance, bot_id, action, target);
        let mechanics = game_data.lock().get_action_mechanics(action);
        if let Some(NetworkedActor::Npc {
            rotation_state,
            timeline,
            ..
        }) = instance.actors.get_mut(&bot_id)
            && let Some(rotation) = &timeline.rotation
        {
            rotation_state.record(rotation, action, target, now, mechanics);
        }
        // Its tank stance: each enemy its area attack reaches is kept on it.
        for enemy in &close {
            if let Some(NetworkedActor::Npc { enmity, .. }) = instance.actors.get_mut(enemy) {
                let top = enmity
                    .iter()
                    .filter(|(hated, _)| **hated != bot_id)
                    .map(|(_, amount)| *amount)
                    .max()
                    .unwrap_or(0);
                let tank = enmity.entry(bot_id).or_insert(0);
                *tank = (*tank).max(top.saturating_add(TANK_STANCE_ENMITY));
            }
        }
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.next_special = now + Duration::from_secs_f32(BOT_PACE);
            bot.pulled.insert(target);
            bot.pulled.extend(close.iter().copied());
        }
    }
    for (bot_id, heal, patient) in heals {
        npc_use_action(network, game_data, instance, bot_id, heal, patient);
        let cast = game_data
            .lock()
            .get_casttime(heal)
            .map(|(_, ms)| ms)
            .unwrap_or(0) as f32
            / 1000.0;
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.next_heal = now + Duration::from_secs_f32(cast.max(BOT_PACE));
        }
    }
}

/// What the tank is attacking, if it's fighting the party: a tank bot's own target, or the player's.
fn tank_target(
    engaged: &[Engaged],
    tank: ObjectId,
    bots: &[(ObjectId, PartyBot)],
    leader_target: Option<ObjectId>,
) -> Option<ObjectId> {
    let is_bot = bots.iter().any(|(id, _)| *id == tank);
    if is_bot {
        // The tank bot's target is the enemy most after it (it pulls them onto itself.)
        engaged
            .iter()
            .find(|enemy| enemy.after == Some(tank))
            .map(|enemy| enemy.id)
            .or_else(|| engaged.first().map(|enemy| enemy.id))
    } else {
        leader_target.filter(|id| engaged.iter().any(|enemy| enemy.id == *id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(id: u32, level: u8, gcd: bool, combo_from: u32) -> ClassAction {
        ClassAction {
            id,
            level,
            gcd,
            combo_from,
            hostile: true,
            single_target: true,
            heals: false,
            area: false,
            range: -1,
            potency: 0,
            dot_seconds: None,
        }
    }

    /// A gladiator tank's area combo is Total Eclipse (around itself, on the GCD, following nothing) and its pull Shield
    /// Lob (its single-target GCD reaching 20); Prominence follows Total Eclipse, so it isn't the starter.
    #[test]
    fn tank_area_combo_and_pull() {
        let mut total_eclipse = action(7381, 6, true, 0);
        total_eclipse.area = true;
        total_eclipse.single_target = false;
        total_eclipse.range = 0;
        let mut prominence = action(16457, 40, true, 7381);
        prominence.area = true;
        prominence.single_target = false;
        let mut shield_lob = action(24, 15, true, 0);
        shield_lob.range = 20;
        let actions = [action(9, 1, true, 0), total_eclipse, prominence, shield_lob];
        assert_eq!(tank_aoe_action(&actions), Some(7381));
        assert_eq!(tank_pull_action(&actions), Some(24));
        assert_eq!(tank_pull_action(&actions[..1]), None);
    }

    /// A gladiator's combo runs to its end (each step before the one it follows), abilities come first, and the
    /// combo's starter is the filler, last.
    #[test]
    fn combos_run_to_their_end() {
        let actions = [
            action(9, 1, true, 0),    // Fast Blade
            action(15, 4, true, 9),   // Riot Blade, after Fast Blade
            action(21, 26, true, 15), // Rage of Halone, after Riot Blade
            action(23, 50, false, 0), // Circle of Scorn (an ability)
        ];
        let rotation = build_rotation(&actions);
        let order: Vec<u32> = rotation.actions.iter().map(|a| a.action).collect();
        assert_eq!(order, vec![23, 21, 15, 9]);
        assert_eq!(rotation.pace, BOT_PACE);
    }

    /// A conjurer's filler is its strongest plain spell (Stone II over Stone), Aero (a damage over time) is re-applied
    /// as it wears off, and heals aren't attacks.
    #[test]
    fn filler_and_damage_over_time() {
        let mut heal = action(120, 2, true, 0);
        heal.hostile = false;
        heal.heals = true;
        let mut stone = action(119, 1, true, 0);
        stone.potency = 140;
        let mut aero = action(121, 4, true, 0);
        aero.potency = 50;
        aero.dot_seconds = Some(30.0);
        let mut stone_ii = action(127, 22, true, 0);
        stone_ii.potency = 200;
        let actions = [stone, aero, stone_ii, heal];
        let rotation = build_rotation(&actions);
        let entries: Vec<(u32, Option<f32>)> = rotation
            .actions
            .iter()
            .map(|a| (a.action, a.refresh))
            .collect();
        assert_eq!(entries, vec![(121, Some(28.0)), (127, None)]);
        assert_eq!(heal_action(&actions), Some(120));
    }

    /// The numbers come from the action scripts themselves.
    #[test]
    fn reads_scripts() {
        let aero = "-- Aero (Lv. 4)\nPOTENCY = 50\nDOT_POTENCY = 30\nDURATION = 30.0\n\nfunction doAction(player, in_combo)\n    effects:damage_over_time(EFFECT_AERO, DURATION, 1)\nend\n";
        assert_eq!(script_facts(aero), (50, Some(30.0)));
        assert_eq!(script_facts("POTENCY = 140\n"), (140, None));
    }

    /// Up to ten bots behind one player all get their own spot, behind them and apart from each other.
    #[test]
    fn duty_finder_fills_the_other_roles() {
        let light_party = PartyRoles {
            members: 4,
            tanks: 1,
            healers: 1,
            melee: 1,
            ranged: 1,
        };
        assert_eq!(roles_left(light_party, ROLE_HEALER), [1, 0, 1, 1]);
        assert_eq!(roles_left(light_party, ROLE_TANK), [0, 1, 1, 1]);
        assert_eq!(roles_left(light_party, ROLE_RANGED), [1, 1, 1, 0]);
        // A DPS takes the other kind's place when theirs is already filled.
        let two_melee = PartyRoles {
            melee: 2,
            ranged: 0,
            ..light_party
        };
        assert_eq!(roles_left(two_melee, ROLE_RANGED), [1, 1, 1, 0]);
    }

    #[test]
    fn formation_spots_apart() {
        let here = Vec3A::new(10.0, 5.0, -3.0);
        let facing = 0.7;
        let spots: Vec<Vec3A> = (0..10).map(|i| formation_spot(here, facing, i)).collect();
        for (i, a) in spots.iter().enumerate() {
            assert!(a.distance(here) >= 2.4, "spot {i} is on the player");
            // Behind: the opposite way from where the player faces.
            let ahead = Vec3A::new(facing.sin(), 0.0, facing.cos());
            assert!((*a - here).dot(ahead) < 0.0, "spot {i} is in front");
            for (j, b) in spots.iter().enumerate().skip(i + 1) {
                assert!(a.distance(*b) >= 1.0, "spots {i} and {j} overlap");
            }
        }
    }

    #[test]
    fn class_names_match_loosely() {
        assert_eq!(normalise("White Mage"), "whitemage");
        assert_eq!(normalise("whitemage"), "whitemage");
        assert_eq!(normalise("WHM"), "whm");
    }

    /// Names keep within retail's 20 characters, as two words.
    #[test]
    fn names_are_two_words() {
        for _ in 0..200 {
            let name = random_name();
            assert!(name.len() <= 20, "{name}");
            assert_eq!(name.split(' ').count(), 2, "{name}");
        }
    }
}
