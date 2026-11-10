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

use crate::collision::Collision;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use glam::Vec3A;
use kawari::{
    common::{
        CharacterMode, HandlerId, JumpState, LogMessageType, MoveAnimationState, MoveAnimationType,
        ObjectId, ObjectTypeId, Position, Rotation, RotationAction,
    },
    config::get_config,
    ipc::chat::TellMessage,
    ipc::zone::{
        ActorControlCategory, BattleNpcSubKind, CharacterDataFlag, CommonSpawn, DisplayFlag,
        InviteType, ObjectKind, OnlineStatus, PartyMemberEntry, PartyUpdateStatus, PlayerSubKind,
        ServerZoneIpcData, ServerZoneIpcSegment, SpawnNpc, SpawnPlayer,
    },
};
use parking_lot::Mutex;
use physis::{
    race::{Gender, Race, Tribe},
    savedata::chardat::CustomizeData,
    sestring::SeString,
};

use crate::{
    BotPartyAction, ClientId, FromServer, GameData, ItemInfoQuery, PartyUpdateTargets, ToServer,
    gamedata::PartyRoles,
    inventory::{EquippedStorage, Inventory, Item},
    lua::{KawariLua, KawariLuaState},
    server::{
        WorldServer,
        actor::{NetworkedActor, NpcState, NpcTarget, set_character_mode, update_actor_hp_mp},
        bot_friends,
        dodge::{self, DODGE_MARGIN, Danger},
        effect::gain_effect_instance,
        glamour,
        instance::Instance,
        modifiers,
        network::{DestinationNetwork, NetworkState},
        npc_behavior::{BEHAVIOR_TICK_SECONDS, NPC_RUN_SPEED, npc_use_action},
        party_npc,
        placement::Placement,
    },
    zone_connection::BaseParameters,
};

/// The level bots are made at: the game's highest, so they're useful anywhere once level synced.
pub const BOT_LEVEL: u8 = 100;

/// How far behind the one they follow bots keep, in yalms, out of combat.
/// The last level of A Realm Reborn's gear, whose item levels are its equip levels (`GameData::best_gear`).
const ARR_MAX_LEVEL: u8 = 50;
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
/// The tank bot answers a little quicker than those players: each reaction time is scaled by this (Bob found the bots
/// slow to react, 2026-10-26; median 1.7 s instead of 2.9.)
const TANK_REACTION_SCALE: f32 = 0.6;
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
/// A party NPC uses its attacks on a target this much further than its fighting distance (the target's hitbox.)
const PARTY_NPC_REACH: f32 = 1.5;
/// How far beyond the edge of the tank's target a melee bot stands behind it (`Placement::behind`), inside its reach.
const MELEE_BEHIND: f32 = 2.0;
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
    /// Its job gauge, as a player's (`server/job_gauge.rs`): its actions' scripts change it and its rotation reads it.
    pub gauge: crate::server::job_gauge::JobGauge,
    /// What it knows at its level that the server can do (its rotation script picks from these.)
    pub known: Vec<KnownAction>,
    pub role: BotRole,
    /// How close it gets to what it attacks.
    pub attack_range: f32,
    /// Its single-target heal (healers), if it has one, and whom it heals: the lowest of the party below this
    /// percent (`HEAL_BELOW_PERCENT`, or a Duty Support member's own, `party_npc::Kit::heal`.)
    pub heal_action: Option<u32>,
    pub heal_below: f32,
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
    /// Since when the zone's collision has stopped it on its way (`walk_like_a_player`), if it has.
    pub walled_since: Option<Instant>,
    /// Since when it has been too far from the one it follows to walk after them (`CATCH_UP_DISTANCE`), if it has.
    pub far_since: Option<Instant>,
    /// Whether it's in the leader's party list: the Duty Finder's are from the start, `!spawn playernpc`'s once
    /// invited (Invite to Party on it.)
    pub in_party: bool,
    /// What it wears (its gear, with the glamour over it), in the equipment's own order, for Examine.
    pub equipped: EquippedStorage,
    /// Its gear's average item level, for Examine.
    pub item_level: u16,
    /// Its made-up account and content ID (the party list's and its SpawnPlayer's): a kept friend's is the same each
    /// time it's called (`bot_friends`), a new bot's comes from its actor ID.
    pub content_id: u64,
    /// The glamour it wears, by name, to wear again when it's kept as a friend.
    pub glamour_name: Option<String>,
    /// When it fell (it lies there, as a player does, until raised or it gets up by itself: `tend_the_fallen`.)
    pub dead_since: Option<Instant>,
    /// When it was first seen offered a revive (Raise), which it accepts a moment later.
    pub revive_seen: Option<Instant>,
    /// Healers: their class's Raise and its range (`raise_action`), and whom they're raising until when.
    pub raise_action: Option<(u32, f32)>,
    pub raising: Option<(ObjectId, Instant)>,
    /// A party NPC (a Duty Support member, `party_npc.rs`): its recorded member and where it is in its kit. None for
    /// the made-up bots (Player NPCs.)
    pub party_npc: Option<party_npc::MemberState>,
    /// Getting out of an enemy's area attack (`dodge.rs`.)
    pub dodge: Option<dodge::Dodge>,
    /// The attack (when it goes off) it last braced for, having nowhere to go, so it braces once for each.
    pub braced_for: Option<Instant>,
    /// When each of its brace actions is ready again (their recast.)
    pub brace_ready: HashMap<u32, Instant>,
    /// How it's shown (`BotLook`), to show it the same way in the next zone.
    pub look_kind: BotLook,
    /// One the Duty Finder added to fill a place (`fill_duty_party`): it stays in the duty when its leader goes, as a
    /// matched stranger does, where a bot of the leader's own party goes with them (`pack_for_travel`.)
    pub duty_filled: bool,
}

impl PartyBot {
    /// How much of a hit meant for a player a party NPC takes (its member's, recorded on retail); 1 for the bots, whose
    /// defences are a player's (`modifiers::apply_hit_on_player`.)
    pub fn damage_taken(&self) -> f32 {
        self.party_npc
            .as_ref()
            .map(|state| state.member.damage_taken)
            .unwrap_or(1.0)
    }

    /// A bot of `role` for `leader`, fighting from `attack_range`, `order`th to join; nothing else set yet.
    pub fn new(leader: ObjectId, role: BotRole, attack_range: f32, order: usize) -> Self {
        let now = Instant::now();
        Self {
            leader,
            gauge: Default::default(),
            known: Vec::new(),
            role,
            attack_range,
            heal_action: None,
            heal_below: HEAL_BELOW_PERCENT,
            next_heal: now,
            aoe_action: None,
            pull_action: None,
            next_special: now,
            answers: HashMap::new(),
            pulled: HashSet::new(),
            order,
            walking_itself: true,
            path: Default::default(),
            reaimed: now,
            walled_since: None,
            far_since: None,
            in_party: false,
            equipped: EquippedStorage::default(),
            item_level: 0,
            content_id: 0,
            glamour_name: None,
            dead_since: None,
            revive_seen: None,
            raise_action: None,
            raising: None,
            party_npc: None,
            dodge: None,
            braced_for: None,
            brace_ready: HashMap::new(),
            look_kind: BotLook::PartyMember,
            duty_filled: false,
        }
    }
}

/// Where a party stands out of combat around the player: retail's Duty Support members kept a sideways diamond around
/// the player, the tank in front, the healer behind and the two damage dealers either side, the player in their own
/// role's place (Bob's Chronofoil capture of 2026-10-23, Sastasha with Duty Support, the player a healer standing still:
/// the tank 2.0 yalms straight ahead, the Lancer 1.0 ahead and 1.72 to one side, the Thaumaturge the same to the other,
/// 183-185 samples each, all within a few hundredths.) The corners, as (yalms ahead, yalms to the melee side) of the
/// diamond's middle: tank, healer, melee, ranged.
const DIAMOND: [(f32, f32); 4] = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.72), (0.0, -1.72)];
const CORNER_TANK: usize = 0;
const CORNER_HEALER: usize = 1;
const CORNER_MELEE: usize = 2;
const CORNER_RANGED: usize = 3;
/// More bots than the diamond has corners stand in rows of three behind it, this far apart.
const EXTRA_ROW_BACK: f32 = 1.6;
const EXTRA_SPACING: f32 = 1.6;

/// The diamond's corners a role takes, first choice first: a damage dealer takes the other side when theirs is taken.
fn corners_for(role: u8) -> &'static [usize] {
    match role {
        ROLE_TANK => &[CORNER_TANK],
        ROLE_HEALER => &[CORNER_HEALER],
        ROLE_RANGED => &[CORNER_RANGED, CORNER_MELEE],
        _ => &[CORNER_MELEE, CORNER_RANGED],
    }
}

/// Each bot's spot around a player of `player_role` (ClassJob Role), as (yalms ahead, yalms to the melee side) of the
/// player: the player takes their role's corner of the diamond, the bots (role, in the order they joined) the rest;
/// the rest stand in rows behind it.
fn diamond_spots(player_role: u8, bots: &[(ObjectId, u8)]) -> HashMap<ObjectId, (f32, f32)> {
    let mut taken = [false; 4];
    let player_corner = corners_for(player_role)[0];
    taken[player_corner] = true;
    let (px, py) = DIAMOND[player_corner];
    let mut spots = HashMap::new();
    let mut extras = 0usize;
    for (id, role) in bots {
        let corner = corners_for(*role)
            .iter()
            .copied()
            .find(|corner| !taken[*corner]);
        let (ahead, side) = match corner {
            Some(corner) => {
                taken[corner] = true;
                DIAMOND[corner]
            }
            None => {
                let row = (extras / 3) as f32;
                let column = (extras % 3) as f32 - 1.0;
                extras += 1;
                (
                    DIAMOND[CORNER_HEALER].0 - EXTRA_ROW_BACK * (row + 1.0),
                    column * EXTRA_SPACING,
                )
            }
        };
        spots.insert(*id, (ahead - px, side - py));
    }
    spots
}

/// The world position `ahead` yalms in front of someone at `position` facing `rotation` and `side` yalms to the melee
/// side (the side retail's Lancer stood: +X when facing +Z.)
fn spot_around(position: Vec3A, rotation: f32, ahead: f32, side: f32) -> Vec3A {
    let (sin, cos) = rotation.sin_cos();
    Vec3A::new(
        position.x + ahead * sin + side * cos,
        position.y,
        position.z + ahead * cos - side * sin,
    )
}

/// How often a walking bot re-aims its path at its moving formation spot: every tick (it was every second, and the
/// bots set off late after the player, Bob 2026-10-26.)
const REAIM_EVERY: Duration = Duration::from_millis(500);
/// The leader has moved when they're this far (yalms, across the ground) from where they stood last tick: then the
/// formation turns to face their way. Turning on the spot leaves it as it was, as retail's Duty Support kept their
/// places while the player looked about (Bob, 2026-10-26.)
const FORMATION_MOVED: f32 = 0.1;

/// A path from `from` to `to` on the navmesh (straight there where it has none: bots walk on regardless.)
fn instance_navmesh_path(navmesh: &crate::navmesh::Navmesh, from: Vec3A, to: Vec3A) -> Vec<Vec3A> {
    if !navmesh.is_available() {
        return vec![to];
    }
    navmesh.calculate_path(from, to, f32::INFINITY)
}

/// A bot this close to its formation spot is there.
const AT_SPOT: f32 = 0.6;
/// How many of the best safe spots a dodger looks for a navmesh path to before it gives up and braces instead.
const DODGE_PATHS_TRIED: usize = 8;
/// The spot moved this far from where it was walking: it sets off for the new one.
const SPOT_MOVED: f32 = 1.0;

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
    /// Its recast in seconds, and its MP cost (the sheet's, PrimaryCostType 3: value × 100.)
    recast: f32,
    mp_cost: u32,
}

/// An action a bot knows, as its rotation script sees it (`bot_rotation::BotView::known`.)
#[derive(Debug, Clone, Copy)]
pub struct KnownAction {
    pub id: u32,
    pub gcd: bool,
    pub recast: f32,
    pub hostile: bool,
    pub mp_cost: u32,
}

/// What an action's script says about it, as far as choosing goes: its `POTENCY`, and its `DURATION` if it puts a
/// damage over time on the target (`damage_over_time`.)
/// A tick's run (at `NPC_RUN_SPEED`) from `here` along `path`, taking off the points it passes: where it got to and the
/// way it faces, on the floor under it, or None if it didn't get anywhere.
fn run_along(
    navmesh: &crate::navmesh::Navmesh,
    collision: Option<&Collision>,
    here: Vec3A,
    path: &mut std::collections::VecDeque<Vec3A>,
    walled_for: Duration,
) -> (Option<(Vec3A, f32)>, bool) {
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
    let facing = f32::atan2(at.x - here.x, at.z - here.z);
    let (at, blocked) = walk_like_a_player(navmesh, collision, here, at, walled_for);
    let flat = Vec3A::new(at.x - here.x, 0.0, at.z - here.z).length();
    if flat < 0.05 {
        return (None, blocked);
    }
    (Some((at, facing)), blocked)
}

/// How long the zone's collision may hold a bot up on its navmesh path before it goes on anyway: something in the
/// collision that isn't there for players. Bob's Copperbell Mines bundle of 2026-10-27: the three bots never left the
/// entrance, stopped a yalm or two in by the zone's collision while he walked off, most likely the duty's entrance
/// barrier (in the collision kawari-navimesh writes, gone for players once the duty starts.)
const WALL_PATIENCE: Duration = Duration::from_secs(2);
/// A bot this far (yalms, any way) from the one it follows, out of combat, for `CATCH_UP_AFTER`, is put beside them:
/// where it can't walk after them (the Copperbell Mines' lift went 30 yalms down and left the bots at the top, Bob's
/// bundle of 2026-10-27.) Retail's Duty Support members are always with the party; how they come down a lift isn't
/// recorded.
const CATCH_UP_DISTANCE: f32 = 25.0;
const CATCH_UP_AFTER: Duration = Duration::from_secs(3);

/// A bot's step from `here` to `to` as a player would take it: on the zone's collision (the floor under it, stopped by
/// walls, down off ledges after the player), or, until the zone has one, on the navmesh's floor under it.
/// Whether it was stopped is returned too. Held up `WALL_PATIENCE` or longer, it goes on to `to` anyway when `to` is
/// walkable ground (on the navmesh, or a floor of the collision within a step of its feet: the wall is one players
/// don't have), on the floor there; elsewhere a wall still stops it (a cliff it would go up.) A gimmick the director
/// put up (a sealed boss room's wall: `Walk::sealed`) always stops it, as it stops a player; another one standing
/// stops it as any wall does (Bob's Copperbell Mines bundle of 2026-10-27: one by the first lift that he walked through,
/// never hidden, held all three until they were put beside him.)
fn walk_like_a_player(
    navmesh: &crate::navmesh::Navmesh,
    collision: Option<&Collision>,
    here: Vec3A,
    mut to: Vec3A,
    walled_for: Duration,
) -> (Vec3A, bool) {
    if let Some(collision) = collision {
        // Down as far as its way goes, not off a ledge its way doesn't (`path_max_drop`.)
        let walk = collision.walk(
            here,
            to,
            crate::server::npc_behavior::path_max_drop(here, to),
        );
        // Said once, as its patience runs out, for the bundles.
        let patience_ran_out = walled_for >= WALL_PATIENCE
            && walled_for < WALL_PATIENCE + Duration::from_secs_f32(BEHAVIOR_TICK_SECONDS);
        if walk.blocked && walled_for >= WALL_PATIENCE {
            if let Some(gimmick) = walk.gimmick
                && walk.sealed
            {
                if patience_ran_out {
                    tracing::info!(
                        "Party bot at {here}: held up by the gimmick of event object {gimmick} (layout ID), put up by the director; it waits, as a player would"
                    );
                }
                return (walk.at, true);
            }
            // Nor past one between it and where it'd go on to (held up by the wall beside a door the director
            // shows shut, it went on through the door: Bob's Haukke Manor bundle 20261103-051833.)
            if collision.crosses_sealed(here, to) {
                if patience_ran_out {
                    tracing::info!(
                        "Party bot at {here}: held up by the zone's collision, with a gimmick the director put up between it and {to}; it waits"
                    );
                }
                return (walk.at, true);
            }
            if let Some(gimmick) = walk.gimmick
                && patience_ran_out
            {
                tracing::info!(
                    "Party bot at {here}: held up by the gimmick of event object {gimmick} (layout ID), which the director hasn't touched; on past it"
                );
            }
            // No deeper than its way goes down (the navmesh's floor 3 yalms down is another storey.)
            let lowest = here.y - crate::server::npc_behavior::path_max_drop(here, to);
            let ground = navmesh
                .ground_under(to)
                .map(|ground| ground.y)
                .filter(|y| *y >= lowest)
                .or_else(|| {
                    collision.floor(
                        to.x,
                        to.z,
                        here.y + collision.climb,
                        here.y - collision.climb,
                    )
                });
            if let Some(y) = ground {
                let settled = collision.settle(Vec3A::new(to.x, y, to.z));
                return (settled, false);
            }
            if patience_ran_out {
                tracing::info!(
                    "Party bot at {here}: held up by the zone's collision, and no ground on the navmesh or the collision at {to} to go on to"
                );
            }
        }
        return (walk.at, walk.blocked);
    }
    if let Some(ground) = navmesh.ground_under(to) {
        to.y = ground.y;
    }
    (to, false)
}

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
    pub fn best_gear(&mut self, classjob: u8, level: u8, wearable: &HashSet<u8>) -> Vec<(u8, u32)> {
        let wears = self.classjob_categories_with(classjob);
        let mut best: HashMap<u8, (u16, u8, u32)> = HashMap::new();
        let mut second_ring: Option<(u16, u8, u32)> = None;
        let rows: Vec<(u32, u8, u8, u8, u16, u8, bool, bool)> = self
            .item_sheet
            .into_iter()
            .flatten_subrows()
            .filter(|(_, row)| row.LevelEquip > 0 && row.EquipSlotCategory > 0)
            .filter(|(_, row)| can_wear(wearable, row.EquipRestriction))
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
        // Up to level 50 (A Realm Reborn's gear) an item's level is its equip level, and nothing better is worn at that
        // level: later items that any level may wear (LevelEquip 1-50) come with item levels far above it. Taken as
        // the best, they gave level-43 Duty Finder bots in the Stone Vigil 5736, 4345 and 3919 HP, where retail's
        // party members there had 2422, 1662 and 1678 at level 41 (Bob's Chronofoil capture 1cefedf4) and a level 41
        // White Mage 1458: his Cure II, 682, took eight casts to fill the tank (bundle 20261110-134029).
        let item_level_cap = if level <= ARR_MAX_LEVEL {
            level as u16
        } else {
            u16::MAX
        };
        for (id, slot, category, level_equip, item_level, rarity, pvp, unique) in rows {
            if pvp || level_equip > level || item_level > item_level_cap {
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

    /// The brace actions (`dodge::BRACE_ACTIONS`) `classjob` has learnt by `level`, with their recast in seconds, in
    /// that list's order.
    pub fn brace_actions(&mut self, classjob: u8, level: u8) -> Vec<(u32, dodge::Brace, f32)> {
        static CATEGORIES: std::sync::LazyLock<Mutex<HashMap<u8, HashSet<u8>>>> =
            std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
        let cached = CATEGORIES.lock().get(&classjob).cloned();
        let categories = match cached {
            Some(categories) => categories,
            None => {
                let categories = self.classjob_categories_with(classjob);
                CATEGORIES.lock().insert(classjob, categories.clone());
                categories
            }
        };
        dodge::BRACE_ACTIONS
            .iter()
            .filter_map(|(id, brace)| {
                let row = self.action_sheet.row(*id)?;
                (row.ClassJobLevel > 0
                    && row.ClassJobLevel <= level
                    && categories.contains(&row.ClassJobCategory))
                .then_some((*id, *brace, row.Recast100ms as f32 / 10.0))
            })
            .collect()
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
                recast: row.Recast100ms as f32 / 10.0,
                mp_cost: if row.PrimaryCostType == 3 {
                    row.PrimaryCostValue as u32 * 100
                } else {
                    0
                },
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

/// The actions that raise the dead: Raise (Conjurer and White Mage), Resurrection (Arcanist, Scholar, Summoner),
/// Ascend (Astrologian) and Egeiro (Sage), by their Action sheet IDs.
const RAISE_ACTIONS: [u32; 4] = [125, 173, 3603, 24287];

/// A healer's raise, of those its class has learnt (and Kawari has a script for), with its range.
fn raise_action(actions: &[ClassAction]) -> Option<(u32, f32)> {
    actions
        .iter()
        .filter(|a| RAISE_ACTIONS.contains(&a.id))
        .max_by_key(|a| (a.level, a.id))
        .map(|a| (a.id, a.range as f32))
}

/// Single-target GCD heals that need something the healer may not have: Afflatus Solace a Lily, Manifestation
/// Seraphism (the job guide, 7.x). The rotation scripts use them (`rotations/WhiteMage.lua`).
const GATED_HEALS: [u32; 2] = [16531, 37015];

/// The healer's heal: its highest level single-target heal on the GCD that needs nothing but MP.
fn heal_action(actions: &[ClassAction]) -> Option<u32> {
    actions
        .iter()
        .filter(|a| a.heals && a.gcd && a.single_target && !GATED_HEALS.contains(&a.id))
        .max_by_key(|a| (a.level, a.id))
        .map(|a| a.id)
}

/// Puts a glamour's pieces over the gear's looks: each in the slot its EquipSlotCategory fills (as the gear's
/// glamour, or as the item where the slot is empty), with its dyes, and the slots it covers (a coat over the legs)
/// emptied. Only the models change; the bot's stats are its gear's.
fn wear_glamour(
    game_data: &mut GameData,
    wearing: &mut Inventory,
    glamour: &glamour::Glamour,
    wearable: &HashSet<u8>,
) {
    // Which pieces went on, and which were left out and why, for the log (Bob's 2026-10-27 bundle: an Elezen bot's
    // feet showed nothing in "keeper of the lilacs", and nothing said which piece it was.)
    let mut worn = Vec::new();
    let mut left_out = Vec::new();
    for piece in &glamour.pieces {
        // A piece only some races or one sex may wear (Miqo'te's own, say) is left out for the rest: there's no model
        // of it for them, so it showed as nothing (Bob's 2026-10-23 bundle: a Viera in a Miqo'te loincloth). The slot
        // keeps its own gear's look.
        let restriction = game_data
            .item_sheet
            .row(piece.item_id)
            .map(|row| row.EquipRestriction)
            .unwrap_or(0);
        if !can_wear(wearable, restriction) {
            left_out.push(format!(
                "{} (EquipRestriction {restriction})",
                piece.item_id
            ));
            continue;
        }
        let Some(row) = game_data
            .equip_slot_category_sheet
            .row(piece.slot_category as u32)
        else {
            left_out.push(format!(
                "{} (no EquipSlotCategory {})",
                piece.item_id, piece.slot_category
            ));
            continue;
        };
        let model = game_data
            .item_sheet
            .row(piece.item_id)
            .map(|row| row.ModelMain)
            .unwrap_or(0);
        worn.push(format!(
            "{} (EquipSlotCategory {}, model {:#x}, stains {:?})",
            piece.item_id, piece.slot_category, model, piece.stains
        ));
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
    tracing::info!(
        "Party bot glamour \"{}\": wears {}; left out {}",
        glamour.name,
        worn.join(", "),
        if left_out.is_empty() {
            "nothing".to_string()
        } else {
            left_out.join(", ")
        }
    );
}

/// Whether an item with this EquipRestriction (an EquipRaceCategory row; 0 for none) may be worn by the race and sex
/// whose wearable rows are `wearable` (`GameData::wearable_restrictions`). An empty set (the sheet couldn't be read)
/// allows everything, as before.
fn can_wear(wearable: &HashSet<u8>, restriction: u8) -> bool {
    restriction == 0 || wearable.is_empty() || wearable.contains(&restriction)
}

impl GameData {
    /// The EquipRaceCategory rows (items' EquipRestriction) this race and sex may wear: each row says which of the
    /// races, and which sexes, may wear its items (all of them, one sex, or one race's own gear.)
    pub fn wearable_restrictions(&mut self, race: Race, gender: Gender) -> HashSet<u8> {
        // A sheet with no text is in no language (its file has no `_en`): read with the server's, it wasn't found, so
        // every bot could wear anything (Bob's 2026-10-27 bundle: a Hyur woman in Miqo'te women's legs, item 3303,
        // EquipRestriction 11.)
        let language = get_config().world.language();
        let Ok(sheet) = icarus::EquipRaceCategory::EquipRaceCategorySheet::read_from(
            &mut self.resource,
            language,
        )
        .or_else(|_| {
            icarus::EquipRaceCategory::EquipRaceCategorySheet::read_from(
                &mut self.resource,
                physis::Language::None,
            )
        }) else {
            tracing::warn!(
                "Party bots: EquipRaceCategory couldn't be read, so they may wear any race's gear"
            );
            return HashSet::new();
        };
        sheet
            .into_iter()
            .flatten_subrows()
            .filter(|(id, row)| {
                let race_ok = match race {
                    Race::Hyur => row.Hyur,
                    Race::Elezen => row.Elezen,
                    Race::Lalafell => row.Lalafell,
                    Race::Miqote => row.Miqote,
                    Race::Roegadyn => row.Roegadyn,
                    Race::AuRa => row.AuRa,
                    Race::Hrothgar => row.Hrothgar,
                    Race::Viera => row.Viera,
                    Race::Unknown => true,
                };
                let sex_ok = match gender {
                    Gender::Male => row.Male,
                    Gender::Female => row.Female,
                };
                *id <= u8::MAX as u32 && race_ok && sex_ok
            })
            .map(|(id, _)| id as u8)
            .collect()
    }
}

/// Retail's Examine window for a party bot (ExamineCharacterInformation), as `(the 640 bytes before the name, the
/// 272 after)`. The layout is read from 13 of these in Bob's Chronofoil captures (2026-09 and 2026-10, numbers only),
/// each matched against the same player's SpawnPlayer and the Item sheet:
///
/// * 0: 4 (always), 1 (once 0), the class, the level; 6: the title (u16); 8: Grand Company and its rank; 32 and 40:
///   the main and off-hand weapon models (as in the spawn); 72: the average item level (u16: 790 at level 100, 7 at
///   level 23).
/// * 80: the 14 equipment slots in the equipment's own order (main hand, off hand, head, body, hands, belt, legs,
///   feet, ears, neck, wrists, right ring, left ring, soul crystal), 40 bytes each: the item (u32), its glamour's item
///   (u32), the crafter's content ID (u64), 1 for high quality (u8), a byte, five materia (u16 type, u16 grade),
///   and the two dyes.
/// * 640: the name (32 bytes, the struct's own field).
/// * 704 (32 into the second part): the look (26 bytes, as in the spawn); 732: the ten gear models (u32 each); 772:
///   their second dyes; 782: the two facewear IDs (u16); 816: the stats, as PlayerStats' first 30 (strength,
///   dexterity, vitality, intelligence, mind, piety, HP, MP, TP, GP, CP, delay, tenacity, attack power, defence,
///   direct hit, evasion, magic defence, critical hit, attack magic potency, healing magic potency, elemental bonus,
///   determination, skill speed, spell speed, haste, then the crafting and gathering four).
///
/// Left 0, as a bot has none: its title (a player's is their spawn's), Grand Company, the free company (16–24, all 0xFF or a value in some), the
/// byte at 10 and the u16 at 50 (57 in most, unknown), crafters and materia.
pub fn examine_bytes(
    common: &CommonSpawn,
    title: u16,
    equipped: Option<&EquippedStorage>,
    item_level: u16,
    stats: Option<&BaseParameters>,
) -> ([u8; 640], [u8; 272]) {
    let mut first = [0u8; 640];
    let mut second = [0u8; 272];

    first[0] = 4;
    first[1] = 1;
    first[2] = common.class_job;
    first[3] = common.level;
    put(&mut first, 32, &le_bytes(&common.main_weapon_model), 8);
    put(&mut first, 40, &le_bytes(&common.sec_weapon_model), 8);
    put(&mut first, 6, &title.to_le_bytes(), 2);
    put(&mut first, 72, &item_level.to_le_bytes(), 2);

    let nothing = EquippedStorage::default();
    let e = equipped.unwrap_or(&nothing);
    let slots = [
        &e.main_hand,
        &e.off_hand,
        &e.head,
        &e.body,
        &e.hands,
        &e.belt,
        &e.legs,
        &e.feet,
        &e.ears,
        &e.neck,
        &e.wrists,
        &e.right_ring,
        &e.left_ring,
        &e.soul_crystal,
    ];
    for (index, item) in slots.iter().enumerate() {
        if item.item_id == 0 {
            continue;
        }
        let at = 80 + index * 40;
        first[at..at + 4].copy_from_slice(&item.item_id.to_le_bytes());
        first[at + 4..at + 8].copy_from_slice(&item.glamour_id.to_le_bytes());
        first[at + 38] = item.stains[0];
        first[at + 39] = item.stains[1];
    }

    put(&mut second, 32, &le_bytes(&common.look), 26);
    put(&mut second, 60, &le_bytes(&common.models), 40);
    second[100..110].copy_from_slice(&common.second_model_stain_ids);
    second[110..112].copy_from_slice(&common.glasses_ids[0].to_le_bytes());
    second[112..114].copy_from_slice(&common.glasses_ids[1].to_le_bytes());
    let Some(stats) = stats else {
        return (first, second);
    };
    let values = [
        stats.strength,
        stats.dexterity,
        stats.vitality,
        stats.intelligence,
        stats.mind,
        stats.piety,
        common.max_health_points,
        common.max_resource_points as u32,
        stats.tp,
        stats.gp,
        stats.cp,
        stats.delay,
        stats.tenacity,
        stats.attack_power,
        stats.defense,
        stats.direct_hit_rate,
        stats.evasion,
        stats.magic_defense,
        stats.critical_hit,
        stats.attack_magic_potency,
        stats.healing_magic_potency,
        stats.elemental_bonus,
        stats.determination,
        stats.skill_speed,
        stats.spell_speed,
        stats.haste,
        stats.craftmanship,
        stats.control,
        stats.gathering,
        stats.perception,
    ];
    for (index, value) in values.iter().enumerate() {
        let at = 144 + index * 4;
        second[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    (first, second)
}

/// `value` as binrw writes it, little-endian.
fn le_bytes<T>(value: &T) -> Vec<u8>
where
    T: for<'a> binrw::BinWrite<Args<'a> = ()>,
{
    let mut cursor = std::io::Cursor::new(Vec::new());
    let _ = value.write_le(&mut cursor);
    cursor.into_inner()
}

/// Copies up to `max` of `bytes` into `into` at `at`.
fn put(into: &mut [u8], at: usize, bytes: &[u8], max: usize) {
    let count = bytes.len().min(max).min(into.len().saturating_sub(at));
    into[at..at + count].copy_from_slice(&bytes[..count]);
}

/// The look's bytes the character creator sets by picking from a list that differs by race, clan and sex: the
/// face, the hairstyle, and the ears or tail (CustomizeData's 5th, 6th and 22nd bytes, as CharaMakeType's menus
/// number them).
const CUSTOMIZE_FACE: u32 = 5;
const CUSTOMIZE_HAIR: u32 = 6;
const CUSTOMIZE_RACE_FEATURE: u32 = 22;

/// One race, clan and sex the character creator offers (a CharaMakeType row), and its choices for the face,
/// hairstyle and ears or tail.
#[derive(Debug, Clone, PartialEq)]
pub struct CreatorLook {
    pub race: u8,
    pub tribe: u8,
    pub gender: u8,
    pub faces: Vec<u8>,
    pub hairstyles: Vec<u8>,
    pub race_features: Vec<u8>,
}

impl GameData {
    /// Every race, clan and sex the character creator offers, with the faces, hairstyles and ears or tails it lists
    /// for each: CharaMakeType's rows, each menu's options for those three bytes (a menu's `Customize` is the byte it
    /// sets), the hairstyles' through CharaMakeCustomize (a hairstyle option is a row of it, whose FeatureID is the
    /// byte). A menu whose options aren't the byte's values (an icon's ID, over 255) counts 1 up to its number of
    /// options. Empty if the sheets can't be read.
    pub fn creator_looks(&mut self) -> Vec<CreatorLook> {
        let language = get_config().world.language();
        let Ok(types) =
            icarus::CharaMakeType::CharaMakeTypeSheet::read_from(&mut self.resource, language)
        else {
            return Vec::new();
        };
        // No text in it, so in no language (as EquipRaceCategory, see `wearable_restrictions`.)
        let customize = icarus::CharaMakeCustomize::CharaMakeCustomizeSheet::read_from(
            &mut self.resource,
            language,
        )
        .or_else(|_| {
            icarus::CharaMakeCustomize::CharaMakeCustomizeSheet::read_from(
                &mut self.resource,
                physis::Language::None,
            )
        })
        .ok();
        let mut looks = Vec::new();
        for (_, row) in types.into_iter().flatten_subrows() {
            if row.Race <= 0 || row.Tribe <= 0 || !(0..=1).contains(&row.Gender) {
                continue;
            }
            let options = |byte: u32| -> Vec<u32> {
                row.CharaMakeStruct
                    .iter()
                    .find(|menu| menu.Customize == byte && menu.SubMenuNum > 0)
                    .map(|menu| {
                        menu.SubMenuParam
                            .iter()
                            .take(menu.SubMenuNum as usize)
                            .copied()
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let as_values = |params: Vec<u32>| -> Vec<u8> {
                if !params.is_empty() && params.iter().all(|value| (1..=255).contains(value)) {
                    params.iter().map(|value| *value as u8).collect()
                } else {
                    (1..=params.len().min(255) as u8).collect()
                }
            };
            let hairstyles = match &customize {
                Some(sheet) => options(CUSTOMIZE_HAIR)
                    .iter()
                    .filter_map(|id| sheet.row(*id))
                    .map(|hair| hair.FeatureID)
                    .filter(|feature| *feature != 0)
                    .collect(),
                None => Vec::new(),
            };
            looks.push(CreatorLook {
                race: row.Race as u8,
                tribe: row.Tribe as u8,
                gender: row.Gender as u8,
                faces: as_values(options(CUSTOMIZE_FACE)),
                hairstyles,
                race_features: as_values(options(CUSTOMIZE_RACE_FEATURE)),
            });
        }
        looks
    }
}

/// A random player look, of any race, clan and sex the character creator offers (`GameData::creator_looks`), with a
/// face, hairstyle and ears or tail from its lists, and random features and colours within the creator's smallest
/// ranges. Without the creator's data (or a list it lacks), the first six races and the choices every one of their
/// clans and sexes has (faces and hairstyles 1-4).
fn random_look(game_data: &mut GameData) -> CustomizeData {
    look_from(&game_data.creator_looks())
}

/// A random look from the creator's races, clans and sexes (`random_look`.)
fn look_from(creator: &[CreatorLook]) -> CustomizeData {
    let pick = |values: &[u8], fallback: std::ops::RangeInclusive<u8>| -> u8 {
        if values.is_empty() {
            fastrand::u8(fallback)
        } else {
            values[fastrand::usize(..values.len())]
        }
    };
    let (race, tribe, gender, face, hair, race_feature_type) = match creator
        .get(fastrand::usize(..creator.len().max(1)))
        .filter(|_| !creator.is_empty())
    {
        Some(look) => (
            Race::from_repr(look.race).unwrap_or_default(),
            Tribe::from_repr(look.tribe).unwrap_or_default(),
            Gender::from_repr(look.gender).unwrap_or_default(),
            pick(&look.faces, 1..=4),
            pick(&look.hairstyles, 1..=4),
            pick(&look.race_features, 1..=4),
        ),
        None => {
            let race = Race::from_repr(fastrand::u8(1..=6)).unwrap_or_default();
            (
                race,
                Tribe::from_repr(race as u8 * 2 - fastrand::u8(0..=1)).unwrap_or_default(),
                if fastrand::bool() {
                    Gender::Male
                } else {
                    Gender::Female
                },
                fastrand::u8(1..=4),
                fastrand::u8(1..=4),
                fastrand::u8(1..=4),
            )
        }
    };
    let eye_colour = fastrand::u8(0..=30);
    CustomizeData {
        race,
        gender,
        age: 1,
        height: fastrand::u8(0..=100),
        tribe,
        face,
        hair,
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
        race_feature_type,
        bust: fastrand::u8(0..=100),
        face_paint: 0,
        face_paint_color: 0,
        ..Default::default()
    }
}

/// Whether a hat (its model, `e<model>`) has a model for this look's race and sex, where that matters: Viera and
/// Hrothgar, whose hats are their own models (`chara/equipment/e<model>/model/c<race code>e<model>_met.mdl`, the race
/// code 1501/1601 for Hrothgar men and women, 1701/1801 for Viera), and which can't wear the rest. Every other race
/// is shown some model of any hat (others' stand in), so for them it's always true.
fn race_has_head_model(game_data: &mut GameData, look: &CustomizeData, model: u16) -> bool {
    let code = match (look.race, look.gender) {
        (Race::Hrothgar, Gender::Male) => 1501,
        (Race::Hrothgar, Gender::Female) => 1601,
        (Race::Viera, Gender::Male) => 1701,
        (Race::Viera, Gender::Female) => 1801,
        _ => return true,
    };
    let path = format!("chara/equipment/e{model:04}/model/c{code:04}e{model:04}_met.mdl");
    game_data.resource.exists(&path)
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
pub(super) fn bot_parameters(
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
/// How a bot is shown to clients. The server treats it as the same owned battle NPC either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotLook {
    /// As another player: a SpawnPlayer like a Duty Finder member's on retail (`player_spawn`). The default, since
    /// 2026-10-23; untested in game.
    Player,
    /// As retail spawns a chocobo companion (sub-kind 3, owned, the name in the spawn's text and no BNpcName,
    /// character data flags 0x1, no display flags): its nameplate shows that name (Bob, in game, 2026-10-21).
    Chocobo,
    /// As retail's Trust members (sub-kind 9, display flags 0x40000): the nameplate shows only the level, as the
    /// client takes the name from a BNpcName (Bob, in game, 2026-10-21).
    PartyMember,
}

impl BotLook {
    pub fn from_word(word: &str) -> Option<Self> {
        match word.to_lowercase().as_str() {
            "player" => Some(Self::Player),
            "chocobo" => Some(Self::Chocobo),
            "partymember" => Some(Self::PartyMember),
            _ => None,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Player => "player",
            Self::Chocobo => "chocobo",
            Self::PartyMember => "partymember",
        }
    }
}

/// How bots are shown when nothing says otherwise (`!spawn playernpc look <kind>` changes it until the server
/// restarts): 0 player, 1 chocobo, 2 party member.
static DEFAULT_LOOK: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub fn default_look() -> BotLook {
    match DEFAULT_LOOK.load(std::sync::atomic::Ordering::Relaxed) {
        1 => BotLook::Chocobo,
        2 => BotLook::PartyMember,
        _ => BotLook::Player,
    }
}

pub fn set_default_look(look: BotLook) {
    let value = match look {
        BotLook::Player => 0,
        BotLook::Chocobo => 1,
        BotLook::PartyMember => 2,
    };
    DEFAULT_LOOK.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// What a bot shown as a player carries in its SpawnPlayer: its made-up account and content ID (the party list's),
/// the leader's world, and the leader's content handler (a duty's, as retail's Duty Finder members have.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShownAsPlayer {
    pub account_id: u64,
    pub content_id: u64,
    pub world_id: u16,
    pub handler_id: HandlerId,
}

/// A bot's SpawnPlayer, from its NPC spawn as it is now, laid out as retail's Duty Finder members' were (Bob's
/// Chronofoil capture of five Sastasha runs, 2026-10-22: the other three members' SpawnPlayer, numbers only): object
/// kind Player, no owner (0xE0000000), battalion 0, ModelChara 0, no BNpcName or base, the duty's content handler,
/// display flags 0x40000 (with INVISIBLE, as they were spawned while the duty loaded; a bot is shown at once), and
/// their own online status (a bot's is Party Member.) Titles, voices and the two unknown bytes after the GM rank are
/// each player's own; a bot has none.
pub fn player_spawn(npc: &SpawnNpc, shown: &ShownAsPlayer) -> SpawnPlayer {
    let mut common = npc.common.clone();
    common.object_kind = ObjectKind::Player(PlayerSubKind::Player);
    common.owner_id = ObjectId::default();
    common.battalion = 0;
    common.model_chara = 0;
    common.base_id = 0;
    common.name_id = 0;
    common.layout_id = 0;
    common.handler_id = shown.handler_id;
    common.display_flags = DisplayFlag::UNK1 | (npc.common.display_flags & DisplayFlag::HIDE_HEAD);
    SpawnPlayer {
        account_id: shown.account_id,
        content_id: shown.content_id,
        current_world_id: shown.world_id,
        home_world_id: shown.world_id,
        online_status: OnlineStatus::PartyMember,
        common,
        ..Default::default()
    }
}

/// An actor ID in the range retail gives players (0x10000000-0x1FFFFFFF, as every player's in Bob's captures), free
/// in this instance, for a bot shown as a player.
fn player_like_actor_id(instance: &Instance) -> ObjectId {
    loop {
        let id = ObjectId(fastrand::u32(0x1000_0000..0x2000_0000));
        if instance.find_actor(id).is_none() {
            return id;
        }
    }
}

pub fn spawn_party_bot(
    data: &mut WorldServer,
    game_data: &mut GameData,
    usable: &HashMap<u32, String>,
    leader: ObjectId,
    class_name: &str,
) -> Result<String, String> {
    // `kind=player`, `kind=chocobo` or `kind=partymember` picks how this one is shown (see BotLook), otherwise the
    // default (`!spawn playernpc look <kind>`).
    let words: Vec<&str> = class_name.split_whitespace().collect();
    let mut look = default_look();
    for word in &words {
        if let Some(kind) = word.strip_prefix("kind=") {
            look = BotLook::from_word(kind).ok_or_else(|| {
                format!("No bot look \"{kind}\": player, chocobo or partymember.")
            })?;
        }
    }
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
    // It follows you at once, and joins your party list when you invite it.
    spawn_bot(
        data, game_data, usable, leader, classjob, BOT_LEVEL, look, false, None,
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
    shown_look: BotLook,
    in_party: bool,
    identity: Option<&bot_friends::SavedBot>,
) -> Result<(ObjectId, String), String> {
    // Shown as a player, it's an owned chocobo-kind NPC to the server, as the chocobo look is.
    let as_chocobo = shown_look != BotLook::PartyMember;
    let (role_column, parent, primary_stat) = game_data
        .classjob_role(classjob)
        .ok_or_else(|| format!("No ClassJob row {classjob}."))?;

    // A kept friend comes back as it was (`bot_friends`); a new one is made up. Chosen first: what it can wear depends
    // on its race and sex.
    let saved_look = identity.and_then(|saved| look_from_hex(&saved.look));
    let look = match saved_look.clone() {
        Some(look) => look,
        None => random_look(game_data),
    };
    let wearable = game_data.wearable_restrictions(look.race, look.gender);

    // Gear, worn on an inventory so the model helpers work on it.
    let gear = game_data.best_gear(classjob, level, &wearable);
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
    // As retail counts it (`EquippedStorage::calculate_item_level`): a one-handed weapon without a shield counts once
    // (Bob's Chronofoil captures: a Gladiator's 5 with nine pieces on, where counting it twice gave 6.)
    let item_level = wearing.equipped.calculate_item_level(game_data);

    let name = identity
        .map(|saved| saved.name.clone())
        .unwrap_or_else(random_name);
    // Its own look: one of the server's glamours for its sex and class, over its gear (`glamour.rs`; none without the
    // list); a kept friend's own again, if the list still has it.
    let glamours = glamour::glamours(game_data);
    let kept_glamour = identity
        .and_then(|saved| saved.glamour.as_ref())
        .and_then(|wanted| glamours.iter().find(|glamour| &glamour.name == wanted));
    let chosen = match identity {
        Some(_) => kept_glamour,
        None => glamour::pick(&glamours, look.gender, classjob, parent),
    };
    let worn_glamour = chosen.map(|glamour| {
        wear_glamour(game_data, &mut wearing, glamour, &wearable);
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
    let raise = if role == BotRole::Healer {
        raise_action(&actions)
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
    // Viera and Hrothgar can't show every hat: one with no model for theirs is hidden, as a player would with the
    // headgear button, rather than shown as a stand-in (Bob's 2026-10-23 bundle: a Viera's glamour hat.)
    let head_model = wearing.legacy_model_ids(game_data)[0].id;
    let hide_head = head_model != 0 && !race_has_head_model(game_data, &look, head_model);
    let shown_flags = if hide_head {
        DisplayFlag::HIDE_HEAD
    } else {
        DisplayFlag::empty()
    };
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
            display_flags: shown_flags
                | if as_chocobo {
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
    let actor_id = if shown_look == BotLook::Player {
        player_like_actor_id(instance)
    } else {
        Instance::generate_actor_id()
    };
    let content_id = identity
        .map(|saved| saved.content_id)
        .unwrap_or(BOT_ID_BASE | actor_id.0 as u64);
    // Shown as a player: the party list's made-up IDs (so the list's entry and the character are one), the leader's
    // world and content handler.
    let shown = match (shown_look, instance.find_actor(leader)) {
        (BotLook::Player, Some(NetworkedActor::Player { spawn, .. })) => Some(ShownAsPlayer {
            account_id: content_id,
            content_id,
            world_id: spawn.home_world_id,
            handler_id: spawn.common.handler_id,
        }),
        _ => None,
    };
    let config = get_config();
    instance.insert_npc(actor_id, spawn, &config, game_data);
    if let Some(NetworkedActor::Npc {
        timeline,
        parameters: npc_parameters,
        state,
        shown_as_player,
        ..
    }) = instance.find_actor_mut(actor_id)
    {
        timeline.autoattack_action_id = autoattack;
        timeline.attack_range = FOLLOW_DISTANCE;
        timeline.hitbox_radius = 0.5;
        timeline.rotation = Some(rotation);
        *npc_parameters = parameters;
        *state = NpcState::Follow;
        *shown_as_player = shown;
    }
    // A new bot (its actor ID may have been another's) starts its rotation afresh.
    crate::server::bot_rotation::forget(actor_id.0);
    instance.party_bots.insert(
        actor_id,
        PartyBot {
            leader,
            gauge: Default::default(),
            known: actions
                .iter()
                .map(|action| KnownAction {
                    id: action.id,
                    gcd: action.gcd,
                    recast: action.recast,
                    hostile: action.hostile,
                    mp_cost: action.mp_cost,
                })
                .collect(),
            role,
            attack_range,
            heal_action: heal,
            heal_below: HEAL_BELOW_PERCENT,
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
            walled_since: None,
            far_since: None,
            in_party,
            equipped: wearing.equipped,
            item_level,
            content_id,
            glamour_name: worn_glamour.clone(),
            dead_since: None,
            revive_seen: None,
            raise_action: raise,
            raising: None,
            party_npc: None,
            dodge: None,
            braced_for: None,
            brace_ready: HashMap::new(),
            look_kind: shown_look,
            duty_filled: false,
        },
    );

    let abbreviation = game_data
        .classjob_sheet
        .row(classjob as u32)
        .map(|row| row.Abbreviation.to_string())
        .unwrap_or_default();
    tracing::info!(
        "Party bot {name} ({abbreviation}, level {level}, {hp} HP) joined {leader} as {actor_id} (shown as {}): gear {item_ids:?}, glamour {worn_glamour:?}, rotation {:?}, heal {heal:?}",
        shown_look.word(),
        actions.iter().map(|a| a.id).collect::<Vec<_>>()
    );
    Ok((actor_id, format!("{name} ({abbreviation})")))
}

/// The base classes Duty Finder bots are picked from (Gladiator, Pugilist, Marauder, Lancer, Archer, Conjurer,
/// Thaumaturge, Arcanist, Rogue): the classes Kawari has action scripts for, levels 1-50. Each one's role is read from
/// the ClassJob sheet.
const DUTY_BOT_CLASSES: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 26, 29];

/// The job a Duty Finder bot of `classjob` plays at `level`: from 30 the base class's job (Gladiator a Paladin, Pugilist
/// a Monk, Marauder a Warrior, Lancer a Dragoon, Archer a Bard, Conjurer a White Mage, Thaumaturge a Black Mage, Arcanist
/// a Summoner, Rogue a Ninja), as retail's Duty Finder players were at 30 and over in Bob's Chronofoil captures (every
/// one of them a job: Paladins, Warriors, Dragoons, Ninjas, Bards, White Mages, Scholars, Black Mages, Summoners...).
/// A healer bot from 30 is a White Mage or, half the time, a Scholar.
fn job_at(classjob: u8, level: u8) -> u8 {
    if level < 30 {
        return classjob;
    }
    match classjob {
        1 => 19,
        2 => 20,
        3 => 21,
        4 => 22,
        5 => 23,
        6 if fastrand::bool() => 28,
        6 => 24,
        7 => 25,
        26 => 27,
        29 => 30,
        other => other,
    }
}

/// The classes of the bots that fill a party of `roles` for a player on `player_classjob`: a role the player plays is
/// theirs, the rest are bots, each of a random class of that role. A DPS player takes a melee or ranged place, their
/// own kind first.
pub fn duty_bot_classes(
    game_data: &mut GameData,
    roles: PartyRoles,
    player_classjob: u8,
    already: &[u8],
) -> Vec<u8> {
    let player_role = game_data
        .classjob_role(player_classjob)
        .map(|(role, _, _)| role)
        .unwrap_or_default();
    let mut left = roles_left(roles, player_role);
    // Roles party NPCs already play (Duty Support members, `party_npc.rs`.)
    for role in already {
        let place = match *role {
            ROLE_TANK => 0,
            ROLE_HEALER => 1,
            ROLE_MELEE if left[2] > 0 => 2,
            ROLE_RANGED if left[3] > 0 => 3,
            _ if left[2] > 0 => 2,
            _ => 3,
        };
        left[place] = left[place].saturating_sub(1);
    }
    let [tanks, healers, melee, ranged] = left;

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
    // The party is never bigger than the duty allows (the player and any party NPCs count too.)
    classes.truncate(roles.members.saturating_sub(1 + already.len() as u8) as usize);
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
/// for the roles they don't play (nor party NPCs, `already`: their ClassJob Roles), level synced to the duty
/// (ClassJobLevelSync). Unsynced duties get none. Returns the bots' actor IDs.
pub fn fill_duty_party(
    data: &mut WorldServer,
    game_data: &mut GameData,
    usable: &HashMap<u32, String>,
    leader: ObjectId,
    content_finder_condition: u16,
    already: &[u8],
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
    for classjob in duty_bot_classes(game_data, roles, player_classjob, already) {
        let classjob = job_at(classjob, level);
        match spawn_bot(
            data,
            game_data,
            usable,
            leader,
            classjob,
            level,
            default_look(),
            true,
            None,
        ) {
            Ok((actor_id, name)) => {
                tracing::info!(
                    "Duty Finder: {name} fills a place in {leader}'s party (level {level})"
                );
                if let Some(bot) = data
                    .find_actor_instance_mut(leader)
                    .and_then(|instance| instance.party_bots.get_mut(&actor_id))
                {
                    bot.duty_filled = true;
                }
                bots.push(actor_id);
            }
            Err(error) => {
                tracing::warn!("Duty Finder: couldn't add a bot of class {classjob}: {error}")
            }
        }
    }
    bots
}

/// A bot of a player's party going with them to another zone: what it takes to spawn the same bot there (its name,
/// class, look and glamour, as a kept friend is called again) and how it's shown.
#[derive(Debug, Clone)]
pub struct TravellingBot {
    pub saved: bot_friends::SavedBot,
    pub look_kind: BotLook,
}

/// The player `leader` is leaving their instance: the bots of their party (`in_party`) go with them, as a party does
/// on retail, and are spawned beside them where they arrive (`arrive`). The Duty Finder's own bots (`duty_filled`), a
/// Duty Support member (a party NPC, the duty's) and bots following outside the party stay behind and go with the
/// instance. Returns the bots packed; `pack_for_travel` is called as the player leaves (`remove_actor_from_instance`),
/// so every way out of a zone takes them along.
pub fn pack_for_travel(
    data: &mut WorldServer,
    network: &mut NetworkState,
    leader: ObjectId,
) -> usize {
    let Some(instance) = data.find_actor_instance_mut(leader) else {
        return 0;
    };
    let mut ids: Vec<(usize, ObjectId)> = instance
        .party_bots
        .iter()
        .filter(|(_, bot)| {
            bot.leader == leader && bot.in_party && !bot.duty_filled && bot.party_npc.is_none()
        })
        .map(|(id, bot)| (bot.order, *id))
        .collect();
    ids.sort_by_key(|(order, _)| *order);
    let mut packed = Vec::new();
    for (_, id) in ids {
        let (Some(bot), Some(actor)) = (instance.party_bots.get(&id), instance.find_actor(id))
        else {
            continue;
        };
        let common = actor.get_common_spawn();
        packed.push(TravellingBot {
            saved: bot_friends::SavedBot {
                content_id: bot.content_id,
                name: common.name.clone(),
                classjob: common.class_job,
                look: look_to_hex(&common.look),
                glamour: bot.glamour_name.clone(),
                added: 0,
            },
            look_kind: bot.look_kind,
        });
        instance.party_bots.remove(&id);
        network.remove_actor(instance, id);
    }
    let count = packed.len();
    if count > 0 {
        tracing::info!("{leader}'s party goes with them: {count} bots");
        data.travelling_bots
            .entry(leader)
            .or_default()
            .extend(packed);
    }
    count
}

/// The player `leader` has loaded into their new instance: the bots of their party that came with them
/// (`pack_for_travel`) are spawned beside them, in the party, at the instance's synced level (a level-synced duty's)
/// or `BOT_LEVEL`. In a duty, no more than its party has room for (`room`, besides the player): the rest wait and come
/// along again when the player next changes zone. Returns the ClassJob Roles of those spawned.
pub fn arrive(
    data: &mut WorldServer,
    game_data: &mut GameData,
    usable: &HashMap<u32, String>,
    leader: ObjectId,
    room: Option<usize>,
) -> Vec<u8> {
    let Some(mut waiting) = data.travelling_bots.remove(&leader) else {
        return Vec::new();
    };
    let Some(instance) = data.find_actor_instance(leader) else {
        data.travelling_bots.insert(leader, waiting);
        return Vec::new();
    };
    let level = instance.synced_level.unwrap_or(BOT_LEVEL);
    let coming = room.unwrap_or(waiting.len()).min(waiting.len());
    let later = waiting.split_off(coming);
    let mut roles = Vec::new();
    for bot in waiting {
        match spawn_bot(
            data,
            game_data,
            usable,
            leader,
            bot.saved.classjob,
            level,
            bot.look_kind,
            true,
            Some(&bot.saved),
        ) {
            Ok((_, name)) => {
                tracing::info!("{name} came with {leader} (level {level})");
                if let Some((role, _, _)) = game_data.classjob_role(bot.saved.classjob) {
                    roles.push(role);
                }
            }
            Err(error) => tracing::warn!("{} couldn't come along: {error}", bot.saved.name),
        }
    }
    if !later.is_empty() {
        tracing::info!(
            "{} of {leader}'s bots wait outside (no room in the party here)",
            later.len()
        );
        data.travelling_bots.insert(leader, later);
    }
    roles
}

/// The party list for a player and the bots that joined them, as retail lists a Duty Finder party (Bob's Chronofoil
/// capture of five Sastasha runs, 2026-10-22): each member with their actor ID, class, level, HP and MP, the zone,
/// `sync_positions` 1, no entity or parent ID. The player comes first and leads. The bots' account and content IDs
/// are made up from their actor IDs (no one else has them.) None when the player has no bots in their party
/// (`in_party`).
pub fn party_list(instance: &Instance, leader: ObjectId) -> Option<Vec<PartyMemberEntry>> {
    let Some(NetworkedActor::Player { spawn: player, .. }) = instance.find_actor(leader) else {
        return None;
    };
    let mut bots: Vec<(&ObjectId, &PartyBot)> = instance
        .party_bots
        .iter()
        .filter(|(_, bot)| bot.leader == leader && bot.in_party)
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
    for (actor_id, bot) in bots {
        if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor(*actor_id) {
            list.push(entry(
                &spawn.common,
                *actor_id,
                bot.content_id,
                bot.content_id,
            ));
        }
    }
    list.truncate(PartyMemberEntry::NUM_ENTRIES);
    Some(list)
}

/// Bots' made-up account and content IDs: their actor ID with this on top.
const BOT_ID_BASE: u64 = 0x00B0_0000_0000_0000;

/// A bot's made-up account and content ID (the party list's, and its SpawnPlayer's.)
pub fn bot_content_id(instance: &Instance, actor_id: ObjectId) -> u64 {
    instance
        .party_bots
        .get(&actor_id)
        .map(|bot| bot.content_id)
        .unwrap_or(BOT_ID_BASE | actor_id.0 as u64)
}

/// A look saved as hex (`bot_friends::SavedBot::look`) read back.
fn look_from_hex(text: &str) -> Option<CustomizeData> {
    use binrw::BinRead;
    let bytes: Vec<u8> = (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(text.get(at..at + 2)?, 16).ok())
        .collect::<Option<_>>()?;
    CustomizeData::read_le(&mut std::io::Cursor::new(bytes)).ok()
}

/// A look as hex, to save.
fn look_to_hex(look: &CustomizeData) -> String {
    le_bytes(look)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The bot of `leader`'s with this made-up content ID, or (content ID 0, as the client's party invite from a
/// character's menu sends) this name.
fn find_bot_by(
    instance: &Instance,
    leader: ObjectId,
    content_id: u64,
    name: &str,
) -> Option<ObjectId> {
    instance
        .party_bots
        .iter()
        .filter(|(_, bot)| bot.leader == leader)
        .find(|(id, bot)| {
            if content_id != 0 {
                bot.content_id == content_id
            } else {
                !name.is_empty()
                    && instance
                        .find_actor(**id)
                        .is_some_and(|actor| actor.get_common_spawn().name == name)
            }
        })
        .map(|(id, _)| *id)
}

/// `leader` doing `what` to `bot`, as a party update names them: the leader executes, the bot is the target.
fn bot_targets(instance: &Instance, leader: ObjectId, bot: ObjectId) -> PartyUpdateTargets {
    let (execute_account_id, execute_content_id, execute_name) = match instance.find_actor(leader) {
        Some(NetworkedActor::Player { spawn, .. }) => (
            spawn.account_id,
            spawn.content_id,
            spawn.common.name.clone(),
        ),
        _ => (0, 0, String::new()),
    };
    let target_name = instance
        .find_actor(bot)
        .map(|actor| actor.get_common_spawn().name.clone())
        .unwrap_or_default();
    PartyUpdateTargets {
        execute_account_id,
        execute_content_id,
        execute_name,
        target_account_id: bot_content_id(instance, bot),
        target_content_id: bot_content_id(instance, bot),
        target_name,
    }
}

/// A party has at most eight.
const PARTY_SIZE: usize = 8;

/// What a bot says back to a tell: `help`, `dismiss`, `leave` and anything else. These are Kawari's own words.
const TELL_HELP: &str =
    "Tell me: \"dismiss\" (I go), \"leave\" (I leave the party and keep following), \"help\".";

/// A player's party actions that concern bots (`ToServer::BotPartyAction`), tells to bots (`ToServer::BotTell`),
/// and Examine (`ToServer::ExamineCharacter`).
///
/// * **Invite to Party** on one of your bots (`!spawn playernpc`'s follow you, but aren't in your party until then):
///   it accepts at once, as a player would: the invite's result, the invite accepted, then the party list with it and
///   a PartyUpdate JoinParty naming it. Not while you're in a party of players (the server keeps those; bots aren't
///   in them), nor past eight. On a bot kept as a friend (`bot_friends`) that isn't with you, from the friend list:
///   it appears beside you (the same name, class, look and glamour, at the duty's synced level in a duty) and joins.
/// * **Kick** a bot: it leaves the party and goes (dismissed); the list without it, and MemberKicked naming it.
/// * **Leave Party** or **Disband Party**: every bot in it goes, and the list empties (DisbandingParty).
/// * **Promote** a bot to leader: refused (a bot can't lead); before, this panicked the server, as these went to
///   the party code with `party_id` 0.
/// * **Add to Friend List** on a bot: it's kept (`bot_friends`), and accepts at once.
/// * **A tell** to a bot: `dismiss`, `leave` or `help`; it answers by tell.
pub fn handle_bot_party_messages(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    lua: Arc<Mutex<KawariLua>>,
    msg: &ToServer,
) -> bool {
    match msg {
        ToServer::BotPartyAction(leader, action) => {
            let leader = *leader;
            let mut data = data.lock();
            let mut network = network.lock();
            let in_players_party = network
                .parties
                .values()
                .any(|party| party.members.iter().any(|m| m.actor_id == leader));
            let Some(instance) = data.find_actor_instance_mut(leader) else {
                return true;
            };
            let leader_content_id = match instance.find_actor(leader) {
                Some(NetworkedActor::Player { spawn, .. }) => spawn.content_id,
                _ => 0,
            };
            match action {
                BotPartyAction::Invite { content_id, name } => {
                    let members = instance
                        .party_bots
                        .values()
                        .filter(|other| other.leader == leader && other.in_party)
                        .count();
                    let full = members + 1 >= PARTY_SIZE;
                    let here = find_bot_by(instance, leader, *content_id, name);
                    // Not here: a bot kept as a friend is called (from the friend list.)
                    let saved = match here {
                        Some(_) => None,
                        None => bot_friends::find(leader_content_id, *content_id, name),
                    };
                    if here.is_none() && saved.is_none() {
                        tracing::info!(
                            "{leader} invited {content_id:#x} ({name}): not a character or one of their bots"
                        );
                        return true;
                    }
                    let already_in = here
                        .and_then(|bot| instance.party_bots.get(&bot))
                        .is_some_and(|bot| bot.in_party);
                    let refused = in_players_party || full || already_in;
                    let reply_id = here
                        .map(|bot| bot_content_id(instance, bot))
                        .or(saved.as_ref().map(|saved| saved.content_id))
                        .unwrap_or(*content_id);
                    network.send_to_by_actor_id(
                        leader,
                        FromServer::InviteCharacterResult(
                            reply_id,
                            if refused {
                                LogMessageType::PlayerAlreadyInAnotherParty as u16
                            } else {
                                0
                            },
                            InviteType::Party,
                            name.clone(),
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                    if refused {
                        tracing::info!(
                            "{leader} invited their bot {name}: refused (in a party of players: {in_players_party}, {members} bots in it, already in: {already_in})"
                        );
                        return true;
                    }
                    let bot = match (here, saved) {
                        (Some(bot), _) => {
                            if let Some(party_bot) = instance.party_bots.get_mut(&bot) {
                                party_bot.in_party = true;
                            }
                            bot
                        }
                        (None, Some(saved)) => {
                            // Called from the friend list: it appears beside the player, in the party.
                            let content = instance.content_finder_condition_id;
                            let usable: HashMap<u32, String> = {
                                let lua = lua.lock();
                                let state = lua.0.app_data_ref::<KawariLuaState>().unwrap();
                                state.action_scripts.clone()
                            };
                            let mut game_data = game_data.lock();
                            let level = if content != 0 {
                                game_data
                                    .find_content_synced_level(content)
                                    .unwrap_or(BOT_LEVEL)
                            } else {
                                BOT_LEVEL
                            };
                            match spawn_bot(
                                &mut data,
                                &mut game_data,
                                &usable,
                                leader,
                                saved.classjob,
                                level,
                                default_look(),
                                true,
                                Some(&saved),
                            ) {
                                Ok((bot, _)) => bot,
                                Err(error) => {
                                    tracing::warn!("Couldn't call {}: {error}", saved.name);
                                    return true;
                                }
                            }
                        }
                        (None, None) => return true,
                    };
                    let Some(instance) = data.find_actor_instance_mut(leader) else {
                        return true;
                    };
                    network.send_to_by_actor_id(
                        leader,
                        FromServer::BotInviteAccepted(
                            reply_id,
                            reply_id,
                            name.clone(),
                            InviteType::Party,
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                    let targets = bot_targets(instance, leader, bot);
                    network.send_to_by_actor_id(
                        leader,
                        FromServer::BotPartyUpdate(
                            party_list(instance, leader),
                            PartyUpdateStatus::JoinParty,
                            targets,
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                    tracing::info!("{leader}'s bot {name} joined their party");
                }
                BotPartyAction::Kick { content_id, name } => {
                    let Some(bot) = find_bot_by(instance, leader, *content_id, "")
                        .or_else(|| find_bot_by(instance, leader, 0, name))
                    else {
                        tracing::info!(
                            "{leader} kicked {content_id:#x} ({name}): none of their bots"
                        );
                        return true;
                    };
                    let targets = bot_targets(instance, leader, bot);
                    instance.party_bots.remove(&bot);
                    network.remove_actor(instance, bot);
                    network.send_to_by_actor_id(
                        leader,
                        FromServer::BotPartyUpdate(
                            party_list(instance, leader),
                            PartyUpdateStatus::MemberKicked,
                            targets,
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                    tracing::info!("{leader} kicked their bot {name}");
                }
                BotPartyAction::Promote { content_id } => {
                    tracing::info!(
                        "{leader} tried to make {content_id:#x} leader: a bot can't lead"
                    );
                }
                BotPartyAction::Leave => {
                    let bots: Vec<ObjectId> = instance
                        .party_bots
                        .iter()
                        .filter(|(_, bot)| bot.leader == leader && bot.in_party)
                        .map(|(id, _)| *id)
                        .collect();
                    if bots.is_empty() {
                        return true;
                    }
                    for bot in &bots {
                        instance.party_bots.remove(bot);
                        network.remove_actor(instance, *bot);
                    }
                    network.send_to_by_actor_id(
                        leader,
                        FromServer::BotParty(None),
                        DestinationNetwork::ZoneClients,
                    );
                }
                BotPartyAction::Befriend { content_id, name } => {
                    let Some(bot) = find_bot_by(instance, leader, *content_id, name) else {
                        tracing::info!(
                            "{leader} sent {content_id:#x} ({name}) a friend request: none of their bots"
                        );
                        return true;
                    };
                    let (Some(party_bot), Some(actor)) =
                        (instance.party_bots.get(&bot), instance.find_actor(bot))
                    else {
                        return true;
                    };
                    let common = actor.get_common_spawn();
                    let saved = bot_friends::SavedBot {
                        content_id: party_bot.content_id,
                        name: common.name.clone(),
                        classjob: common.class_job,
                        look: look_to_hex(&common.look),
                        glamour: party_bot.glamour_name.clone(),
                        added: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|time| time.as_secs() as u32)
                            .unwrap_or_default(),
                    };
                    let reply_id = saved.content_id;
                    let new = bot_friends::add(leader_content_id, saved);
                    network.send_to_by_actor_id(
                        leader,
                        FromServer::InviteCharacterResult(
                            reply_id,
                            0,
                            InviteType::FriendList,
                            name.clone(),
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                    network.send_to_by_actor_id(
                        leader,
                        FromServer::BotInviteAccepted(
                            reply_id,
                            reply_id,
                            name.clone(),
                            InviteType::PendingFriendList,
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                    tracing::info!(
                        "{leader} kept their bot {name} as a friend (already: {})",
                        !new
                    );
                }
            }
            true
        }
        ToServer::BotTell(sender, recipient, text) => {
            let sender = *sender;
            let mut data = data.lock();
            let mut network = network.lock();
            let Some(instance) = data.find_actor_instance_mut(sender) else {
                return true;
            };
            let Some(bot) = find_bot_by(instance, sender, 0, recipient) else {
                network.send_to_by_actor_id(
                    sender,
                    FromServer::BotTellNotFound(recipient.clone()),
                    DestinationNetwork::ChatClients,
                );
                return true;
            };
            let content_id = bot_content_id(instance, bot);
            let command = text.trim().to_lowercase();
            let reply = match command.as_str() {
                "dismiss" => {
                    let in_party = instance
                        .party_bots
                        .get(&bot)
                        .is_some_and(|bot| bot.in_party);
                    let targets = bot_targets(instance, sender, bot);
                    instance.party_bots.remove(&bot);
                    network.remove_actor(instance, bot);
                    if in_party {
                        network.send_to_by_actor_id(
                            sender,
                            FromServer::BotPartyUpdate(
                                party_list(instance, sender),
                                PartyUpdateStatus::MemberLeftParty,
                                targets,
                            ),
                            DestinationNetwork::ZoneClients,
                        );
                    }
                    "Goodbye!".to_string()
                }
                "leave" => {
                    let was_in = instance
                        .party_bots
                        .get_mut(&bot)
                        .map(|bot| std::mem::replace(&mut bot.in_party, false))
                        .unwrap_or_default();
                    if was_in {
                        let targets = bot_targets(instance, sender, bot);
                        network.send_to_by_actor_id(
                            sender,
                            FromServer::BotPartyUpdate(
                                party_list(instance, sender),
                                PartyUpdateStatus::MemberLeftParty,
                                targets,
                            ),
                            DestinationNetwork::ZoneClients,
                        );
                        "I've left the party; I'll keep following you.".to_string()
                    } else {
                        "I'm not in your party.".to_string()
                    }
                }
                _ => TELL_HELP.to_string(),
            };
            tracing::info!("{sender} told their bot {recipient}: {command:?}");
            // The answer, as a tell from the bot (sent from it, as tells come from their sender.)
            let world_id = get_config().world.world_id;
            network.send_to_by_actor_id(
                sender,
                FromServer::TellMessageReceived(
                    bot,
                    TellMessage {
                        sender_account_id: content_id,
                        sender_content_id: content_id,
                        sender_world_id: world_id,
                        sender_name: recipient.clone(),
                        message: SeString::from(reply.as_str()),
                        ..Default::default()
                    },
                ),
                DestinationNetwork::ChatClients,
            );
            true
        }
        ToServer::ExamineCharacter(examiner, target) => {
            let data = data.lock();
            let mut network = network.lock();
            let Some(instance) = data.find_actor_instance(*examiner) else {
                return true;
            };
            let (first, second, name) = match (
                instance.find_actor(*target),
                instance.party_bots.get(target),
            ) {
                (
                    Some(NetworkedActor::Npc {
                        spawn, parameters, ..
                    }),
                    Some(bot),
                ) => {
                    let (first, second) = examine_bytes(
                        &spawn.common,
                        0,
                        Some(&bot.equipped),
                        bot.item_level,
                        Some(parameters),
                    );
                    (first, second, spawn.common.name.clone())
                }
                // A player: what their spawn shows (their gear's items and stats aren't on the server's side.)
                (Some(NetworkedActor::Player { spawn, .. }), _) => {
                    let (first, second) =
                        examine_bytes(&spawn.common, spawn.title_id, None, 0, None);
                    (first, second, spawn.common.name.clone())
                }
                _ => return true,
            };
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ExamineCharacterInformation {
                unk1: first,
                name,
                unk2: second,
            });
            // As retail's: from the one examined.
            network.send_to_by_actor_id(
                *examiner,
                FromServer::PacketSegment(ipc, *target),
                DestinationNetwork::ZoneClients,
            );
            true
        }
        _ => false,
    }
}

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

/// Sends away the duty's own members of `leader`'s party, the bots the Duty Finder added (`duty_filled`) and Duty
/// Support's members (party NPCs), as the duty ends for them; the bots of their own party stay with them. Returns how
/// many went.
pub fn dismiss_duty_members(
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
        .filter(|(_, bot)| bot.leader == leader && (bot.duty_filled || bot.party_npc.is_some()))
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
    /// Its hitbox radius (its timeline's): how far from its middle its edge is.
    hitbox: f32,
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
    let mut left_behind = HashSet::new();
    for id in gone {
        if let Some(bot) = instance.party_bots.remove(&id)
            && bot.in_party
            && !instance.actors.contains_key(&bot.leader)
        {
            left_behind.insert(bot.leader);
        }
        if instance.actors.contains_key(&id) {
            network.lock().remove_actor(instance, id);
        }
    }
    // A player who left the zone without their bots (they don't follow): their party list empties.
    for leader in left_behind {
        network.lock().send_to_by_actor_id(
            leader,
            FromServer::BotParty(None),
            DestinationNetwork::ZoneClients,
        );
    }

    tend_the_fallen(network, instance, now);
    raise_the_fallen(network, game_data, instance, now);
    party_npc::update_yells(network, game_data, instance, now);

    let leaders: HashSet<ObjectId> = instance.party_bots.values().map(|bot| bot.leader).collect();
    for leader in leaders {
        update_party(network, game_data, instance, leader, now);
    }
}

/// How long a bot takes to accept a Raise (a player's choice; this is a moment.)
const RAISE_ACCEPT_DELAY: Duration = Duration::from_secs(2);
/// How long a bot lies dead before it gets up by itself beside the one it follows.
const GET_UP_AFTER: Duration = Duration::from_secs(30);
/// Status: a revive offered by Raise (`EffectsBuilder::offer_revive`), Weakness and Brink of Death (100 s), and the
/// 5 s status retail gives a raised player: as the server's `revive_in_place` gives a player.
const STATUS_REVIVE_OFFER: u16 = 148;
const STATUS_WEAKNESS: u16 = 43;
const STATUS_BRINK_OF_DEATH: u16 = 44;
const STATUS_RAISED: u16 = 418;
const WEAKNESS_DURATION: f32 = 100.0;
const RAISED_DURATION: f32 = 5.0;

/// Dead bots: a bot lies where it fell, as a player does (`kill_actor` doesn't fade it out.) Raised (someone's Raise
/// offered it a revive, status 148), it accepts after `RAISE_ACCEPT_DELAY` and gets up there as a raised player
/// does: 20 % of its HP and MP, Weakness (Brink of Death if already weakened) and status 418, the raise's stand-up
/// (ZoneIn with its raise animation, then its mode back to normal.) Not raised within `GET_UP_AFTER`, it gets up by
/// itself beside the one it follows, at full HP and MP, as a player returning would, and carries on.
fn tend_the_fallen(network: &Arc<Mutex<NetworkState>>, instance: &mut Instance, now: Instant) {
    let ids: Vec<ObjectId> = instance.party_bots.keys().copied().collect();
    for id in ids {
        let Some(actor) = instance.find_actor(id) else {
            continue;
        };
        let dead = actor.get_common_spawn().mode == CharacterMode::Dead
            || actor.get_common_spawn().health_points == 0;
        let offered = actor
            .status_effects()
            .is_some_and(|effects| effects.get(STATUS_REVIVE_OFFER).is_some());
        let Some(bot) = instance.party_bots.get_mut(&id) else {
            continue;
        };
        if !dead {
            bot.dead_since = None;
            bot.revive_seen = None;
            continue;
        }
        let since = *bot.dead_since.get_or_insert(now);
        if offered && bot.revive_seen.is_none() {
            bot.revive_seen = Some(now);
        }
        let leader = bot.leader;
        if bot
            .revive_seen
            .is_some_and(|seen| now.duration_since(seen) >= RAISE_ACCEPT_DELAY)
        {
            bot.dead_since = None;
            bot.revive_seen = None;
            raise_bot(network, instance, id);
        } else if now.duration_since(since) >= GET_UP_AFTER {
            bot.dead_since = None;
            bot.revive_seen = None;
            get_up_beside(network, instance, id, leader);
        }
    }
}

/// Healer bots raise the dead of their party (the one they follow, and that player's other bots), in combat or out of
/// it, before healing: the nearest within their raise's range that hasn't a revive offered already and isn't being
/// raised by another bot. A raised player accepts as from anyone; a bot accepts by itself (`tend_the_fallen`.)
fn raise_the_fallen(
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
    now: Instant,
) {
    let is_dead = |instance: &Instance, id: ObjectId| {
        instance.find_actor(id).is_some_and(|actor| {
            let common = actor.get_common_spawn();
            common.mode == CharacterMode::Dead || common.health_points == 0
        })
    };
    let offered = |instance: &Instance, id: ObjectId| {
        instance
            .find_actor(id)
            .and_then(|actor| actor.status_effects())
            .is_some_and(|effects| effects.get(STATUS_REVIVE_OFFER).is_some())
    };
    let healers: Vec<(ObjectId, ObjectId, u32, f32)> = instance
        .party_bots
        .iter()
        .filter_map(|(id, bot)| {
            let (raise, range) = bot.raise_action?;
            (now >= bot.next_heal && bot.raising.is_none_or(|(_, until)| until <= now))
                .then_some((*id, bot.leader, raise, range))
        })
        .collect();
    for (healer, leader, raise, range) in healers {
        let Some(NetworkedActor::Npc { spawn, cast, .. }) = instance.find_actor(healer) else {
            continue;
        };
        if spawn.common.health_points == 0 || cast.is_some_and(|cast| cast.until > now) {
            continue;
        }
        let here = spawn.common.position.0;
        let taken: Vec<ObjectId> = instance
            .party_bots
            .values()
            .filter_map(|bot| bot.raising)
            .filter(|(_, until)| *until > now)
            .map(|(target, _)| target)
            .collect();
        let mut fallen: Vec<ObjectId> = vec![leader];
        fallen.extend(
            instance
                .party_bots
                .iter()
                .filter(|(id, bot)| bot.leader == leader && **id != healer)
                .map(|(id, _)| *id),
        );
        let target = fallen
            .into_iter()
            .filter(|id| is_dead(instance, *id) && !offered(instance, *id) && !taken.contains(id))
            .filter_map(|id| {
                let distance = instance.find_actor(id)?.position().0.distance(here);
                (distance <= range.max(1.0)).then_some((id, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id);
        let Some(target) = target else {
            continue;
        };
        npc_use_action(network, game_data, instance, healer, raise, target);
        let cast = game_data
            .lock()
            .get_casttime(raise)
            .map(|(_, ms)| ms)
            .unwrap_or(0) as f32
            / 1000.0;
        if let Some(bot) = instance.party_bots.get_mut(&healer) {
            bot.raising = Some((target, now + Duration::from_secs_f32(cast + 1.0)));
            bot.next_heal = now + Duration::from_secs_f32(cast.max(BOT_PACE));
        }
        tracing::info!("Party bot {healer} raises {target} ({raise}, {cast} s)");
    }
}

/// Where someone who accepts a Raise gets up: beside whoever raised them (the source of their revive offer), where
/// that one stands as they accept. Retail warped the raised player onto their raiser (within a yalm) in each of 44
/// raises in Bob's Chronofoil captures, however far off they had died (up to 21 yalms in combat); `None` if the
/// raiser isn't here, when they get up where they lie.
pub fn raiser_position(instance: &Instance, raised: ObjectId) -> Option<Position> {
    let raiser = instance
        .find_actor(raised)?
        .status_effects()?
        .get(STATUS_REVIVE_OFFER)?
        .source_actor_id;
    if raiser == raised {
        return None;
    }
    instance.find_actor(raiser).map(|actor| actor.position())
}

/// A bot raised beside its raiser (see `tend_the_fallen`, `raiser_position`.)
fn raise_bot(network: &Arc<Mutex<NetworkState>>, instance: &mut Instance, id: ObjectId) {
    let beside = raiser_position(instance, id);
    let Some(actor) = instance.find_actor_mut(id) else {
        return;
    };
    if let Some(position) = beside {
        let rotation = actor.rotation();
        actor.get_common_spawn_mut().position = position;
        network.lock().send_in_range_instance(
            id,
            instance,
            FromServer::ActorMove(
                id,
                position,
                rotation,
                MoveAnimationType::empty(),
                MoveAnimationState::None,
                JumpState::NoneOrFalling,
                Some((NPC_RUN_SPEED * 10.0) as u8),
            ),
            DestinationNetwork::ZoneClients,
        );
    }
    let Some(actor) = instance.find_actor_mut(id) else {
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
    stand_up(network, instance, id);
    let penalty = if weakened {
        STATUS_BRINK_OF_DEATH
    } else {
        STATUS_WEAKNESS
    };
    for (status, duration) in [
        (penalty, WEAKNESS_DURATION),
        (STATUS_RAISED, RAISED_DURATION),
    ] {
        gain_effect_instance(
            network.clone(),
            ClientId::default(),
            instance,
            id,
            status,
            0,
            duration,
            ObjectId(0xE000_0000),
            true,
        );
    }
    update_actor_hp_mp(network.clone(), instance, id);
    tracing::info!("Party bot {id} was raised");
}

/// The raise's stand-up for everyone near (ZoneIn with its raise animation, as others see a raised player), then the
/// bot's mode back to normal and its state to following.
fn stand_up(network: &Arc<Mutex<NetworkState>>, instance: &mut Instance, id: ObjectId) {
    let mut network = network.lock();
    network.send_ac_in_range_inclusive_instance(
        instance,
        id,
        ActorControlCategory::ZoneIn {
            warp_finish_anim: 1,
            raise_anim: 1,
            unk1: 0,
        },
    );
    set_character_mode(instance, &mut network, id, CharacterMode::Normal, 0, true);
    if let Some(NetworkedActor::Npc {
        state,
        navmesh_path,
        enmity,
        ..
    }) = instance.find_actor_mut(id)
    {
        *state = NpcState::Follow;
        navmesh_path.clear();
        enmity.clear();
    }
}

/// A bot that wasn't raised gets up beside the one it follows (see `tend_the_fallen`): taken away from where it lay
/// and spawned again there, alive, at full HP and MP.
pub(super) fn get_up_beside(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    id: ObjectId,
    leader: ObjectId,
) {
    let Some(spot) = instance.find_actor(leader).map(|actor| {
        let mut position = actor.position();
        let behind = actor.rotation() + std::f32::consts::PI;
        position.0.x += FOLLOW_DISTANCE * behind.sin();
        position.0.z += FOLLOW_DISTANCE * behind.cos();
        (position, actor.rotation())
    }) else {
        return;
    };
    let Some(mut actor) = instance.actors.remove(&id) else {
        return;
    };
    // Clients forget it where it lay; it's spawned for them again, standing, where it gets up.
    network.lock().remove_actor(instance, id);
    if let NetworkedActor::Npc {
        spawn,
        state,
        navmesh_path,
        enmity,
        status_effects,
        cast,
        ..
    } = &mut actor
    {
        spawn.common.mode = CharacterMode::Normal;
        spawn.common.mode_arg = 0;
        spawn.common.health_points = spawn.common.max_health_points;
        spawn.common.resource_points = spawn.common.max_resource_points;
        spawn.common.position = spot.0;
        spawn.common.rotation = spot.1;
        *state = NpcState::Follow;
        navmesh_path.clear();
        enmity.clear();
        *cast = None;
        status_effects.remove(STATUS_REVIVE_OFFER);
    }
    instance.actors.insert(id, actor);
    if let Some(bot) = instance.party_bots.get_mut(&id) {
        bot.path.clear();
        bot.walking_itself = true;
    }
    update_actor_hp_mp(network.clone(), instance, id);
    tracing::info!("Party bot {id} got up by itself beside {leader}");
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

    // Everything fighting the party: any NPC (not a bot) with one of them on its enmity list, unless the director has
    // made it untargetable (`Instance::untargetable_npcs`: Titan as he leaps and behind his Heart.)
    let engaged: Vec<Engaged> = instance
        .actors
        .iter()
        .filter(|(id, _)| !instance.party_bots.contains_key(id))
        .filter(|(id, _)| !instance.untargetable_npcs.contains(id))
        .filter_map(|(id, actor)| match actor {
            NetworkedActor::Npc {
                enmity,
                spawn,
                timeline,
                ..
            } if spawn.common.health_points > 0
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
                    hitbox: timeline.hitbox_radius,
                })
            }
            _ => None,
        })
        .collect();
    let is_engaged = |id: ObjectId| engaged.iter().any(|enemy| enemy.id == id);

    // Whom the healer helps first: the lowest of the party, if they're below its line (`PartyBot::heal_below`.)
    let hurt = members
        .iter()
        .filter_map(|id| instance.find_actor(*id).map(|actor| (*id, actor)))
        .filter(|(_, actor)| actor.get_common_spawn().health_points > 0)
        .map(|(id, actor)| (id, health_percent(actor)))
        .min_by(|a, b| a.1.total_cmp(&b.1));

    // Where everyone is and faces, and each bot's place behind the one it follows.
    let where_is: HashMap<ObjectId, (Vec3A, f32)> = members
        .iter()
        .filter_map(|id| {
            instance
                .find_actor(*id)
                .map(|actor| (*id, (actor.position().0, actor.rotation())))
        })
        .collect();
    // The way the formation faces: the leader's, taken only as they move (`FORMATION_MOVED`).
    let formation_facing = where_is.get(&leader).copied().map(|(position, rotation)| {
        let entry = instance
            .formation_facings
            .entry(leader)
            .or_insert((position, rotation));
        let moved = Vec3A::new(position.x - entry.0.x, 0.0, position.z - entry.0.z).length();
        if moved > FORMATION_MOVED {
            entry.1 = rotation;
        }
        entry.0 = position;
        entry.1
    });
    let follows = |bot_id: ObjectId| {
        if Some(bot_id) == tank {
            leader
        } else {
            tank.unwrap_or(leader)
        }
    };
    // Out of combat everyone keeps their place in the diamond around the player (`diamond_spots`), the player in
    // their own role's corner.
    let spots = {
        let mut by_order: Vec<&(ObjectId, PartyBot)> = bots.iter().collect();
        by_order.sort_by_key(|(_, bot)| bot.order);
        let roles: Vec<(ObjectId, u8)> = by_order
            .iter()
            .map(|(id, bot)| {
                let role = match bot.role {
                    BotRole::Tank => ROLE_TANK,
                    BotRole::Healer => ROLE_HEALER,
                    BotRole::Dps if bot.attack_range > MELEE_RANGE => ROLE_RANGED,
                    BotRole::Dps => ROLE_MELEE,
                };
                (*id, role)
            })
            .collect();
        let player_role = game_data
            .lock()
            .classjob_role(leader_classjob)
            .map(|(role, _, _)| role)
            .unwrap_or(ROLE_MELEE);
        diamond_spots(player_role, &roles)
    };

    let mut heals = Vec::new();
    let mut provokes = Vec::new();
    let mut specials: Vec<(ObjectId, u32, ObjectId, Vec<ObjectId>)> = Vec::new();
    let mut tank_answers: Vec<(ObjectId, HashMap<ObjectId, Instant>)> = Vec::new();
    let mut repulls: Vec<(ObjectId, ObjectId)> = Vec::new();
    let mut walks = Vec::new();
    // Whether the zone's collision stopped each walking bot this tick, and since when (`walk_like_a_player`.)
    let mut walled: Vec<(ObjectId, Option<Instant>)> = Vec::new();
    // Since when each walking bot has been too far behind, and those put beside the one they follow this tick.
    let mut far: Vec<(ObjectId, Option<Instant>)> = Vec::new();
    let mut catch_ups: Vec<(ObjectId, Vec3A, f32)> = Vec::new();
    let mut fighting = Vec::new();
    // Party NPCs: their kits' actions (actor, action, target), their states after this tick and their lines.
    let mut kit_actions: Vec<(ObjectId, u32, ObjectId)> = Vec::new();
    let mut npc_states: Vec<(ObjectId, party_npc::MemberState)> = Vec::new();
    let mut lines: Vec<(ObjectId, u32)> = Vec::new();
    // Enemies' area attacks being cast: everyone in one gets out of it (`dodge.rs`.)
    let dangers: Vec<Danger> = dodge::dangers(instance, &mut game_data.lock(), now);
    // The waves of them it tries to be out of, all first, then the soonest ones (`dodge::in_turn`.)
    let in_turn = dodge::in_turn(&dangers);
    // A boss room's floor while its boss is fought: no spot off it counts (`dodge::arena`.)
    let arena = dodge::arena(instance);
    let mut dodges: Vec<(ObjectId, Option<dodge::Dodge>)> = Vec::new();
    // Each member's role and its place among those of that role, for the spots the duty's director authored
    // (`director:party_spots`, `director:party_positions`: retail's Duty Support members', `dodge::authored_spot`.)
    let authoring = !instance.party_spots.is_empty() || instance.party_cue.is_some();
    let mut roles: HashMap<ObjectId, (&'static str, usize)> = HashMap::new();
    if authoring {
        let mut ids: Vec<ObjectId> = bots.iter().map(|(id, _)| *id).collect();
        ids.sort_by_key(|id| id.0);
        let mut counts: HashMap<&'static str, usize> = HashMap::new();
        for id in ids {
            let Some(actor) = instance.find_actor(id) else {
                continue;
            };
            let classjob = actor.get_common_spawn().class_job;
            let role = game_data
                .lock()
                .classjob_role(classjob)
                .map(|(role, _, _)| party_npc::role_name(role))
                .unwrap_or_default();
            let nth = counts.entry(role).or_default();
            roles.insert(id, (role, *nth));
            *nth += 1;
        }
    }
    // The party where it stands (for bracing: who an attack catches, and who's most hurt.)
    let allies: Vec<(ObjectId, Vec3A, f32)> = members
        .iter()
        .filter_map(|id| instance.find_actor(*id).map(|actor| (*id, actor)))
        .filter(|(_, actor)| actor.get_common_spawn().health_points > 0)
        .map(|(id, actor)| (id, actor.position().0, health_percent(actor)))
        .collect();
    // Braces used this tick: (member, the attack it's for, the action, its recast.)
    let mut braces: Vec<(ObjectId, Instant, u32, f32)> = Vec::new();
    let collision = instance.collision.clone();
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
            status_effects,
            ..
        }) = instance.actors.get_mut(bot_id)
        else {
            continue;
        };
        if spawn.common.health_points == 0 || *state == NpcState::Dead {
            continue;
        }
        // Stunned, asleep or gaoled (the Navel's Granite Gaol): it does nothing until that's over, as retail's gaoled
        // Duty Support members (`modifiers::is_incapacitated`.)
        if modifiers::is_incapacitated(status_effects) {
            continue;
        }
        let follow = follows(*bot_id);
        let casting = cast.is_some_and(|cast| cast.until > now);

        // An enemy's area attack over it: out of it, after a moment (retail's Duty Support members, `dodge.rs`), to
        // the nearest spot out of every one being cast, as near its target (or the player) as that allows; it waits
        // there, still acting if what it's after is in reach, until a moment after the last has gone off. With
        // nowhere out of all of them, out of the ones going off soonest (waiting where it is if it's already out of
        // those), and on to the next once they have; never off a boss room's floor.
        let here = spawn.common.position.0;
        let threats: Vec<&Danger> = dangers
            .iter()
            .filter(|danger| danger.catches(here, DODGE_MARGIN))
            .collect();
        let mut dodge = bot.dodge;
        // Where its duty's director has it go (`dodge::authored_spot`), whether the cast would catch it or not, if
        // a path there along the navmesh reaches it; else it dodges by the shapes, as before.
        let authored = roles.get(bot_id).and_then(|(role, nth)| {
            let (at, until, react) = dodge::authored_spot(
                &instance.party_spots,
                instance.party_cue.as_ref(),
                &dangers,
                role,
                *nth,
                now,
            )?;
            let flat = |a: Vec3A, b: Vec3A| Vec3A::new(a.x - b.x, 0.0, a.z - b.z).length();
            let to = Vec3A::new(at.x, here.y, at.y);
            if flat(here, to) <= AT_SPOT {
                return Some((None, until, react));
            }
            let path = instance_navmesh_path(navmesh, here, to);
            let end = path.last().copied()?;
            (flat(end, to) <= AT_SPOT).then_some((Some(end), until, react))
        });
        if let Some((to, until, react)) = authored {
            let entry = dodge.get_or_insert(dodge::Dodge {
                to: None,
                set_off: dodge::authored_set_off(now, react, fastrand::f32()),
                until,
                holding: false,
            });
            entry.until = entry.until.max(until);
            // Already there: it holds where it stands.
            if to.is_some() || entry.to.is_none() {
                entry.to = to;
            }
        } else if !threats.is_empty() {
            let until = threats
                .iter()
                .map(|danger| danger.until)
                .max()
                .unwrap_or(now);
            let rather = match navmesh_target {
                Some(NpcTarget::Actor(target)) => engaged
                    .iter()
                    .find(|enemy| enemy.id == *target)
                    .map(|enemy| enemy.position),
                _ => None,
            }
            .or_else(|| where_is.get(&leader).map(|(position, _)| *position))
            .unwrap_or(here);
            // Only somewhere on the map: on the navmesh, with a path there along it (Bob, 2026-10-24); without a
            // navmesh nowhere is known to be, and it braces instead (below.)
            let flat = |a: Vec3A, b: Vec3A| Vec3A::new(a.x - b.x, 0.0, a.z - b.z).length();
            let walkable = |point: Vec3A| {
                navmesh.is_available()
                    && navmesh.ground_under(point).is_some()
                    && arena.is_none_or(|arena| arena.holds(point))
            };
            let reachable = |to: Vec3A| {
                instance_navmesh_path(navmesh, here, to)
                    .last()
                    .is_some_and(|end| flat(*end, to) <= AT_SPOT)
            };
            // The most waves it can be out of: where it stands, the spot it was going to, or a new one.
            let choice = dodge::choose(
                here,
                rather,
                &in_turn,
                dodge.and_then(|dodge| dodge.to),
                walkable,
                |spots| {
                    spots
                        .into_iter()
                        .take(DODGE_PATHS_TRIED)
                        .find(|to| reachable(*to))
                },
            );
            let (spot, waiting) = match choice {
                dodge::Choice::Go(to) => (Some(to), false),
                dodge::Choice::Wait => (None, true),
                dodge::Choice::Nowhere => (None, false),
            };
            if spot.is_none() && !waiting && bot.braced_for.is_none_or(|braced| braced < until) {
                // Nowhere to go: it stays and shields what it can (`dodge::choose_brace`.)
                let caught: Vec<&(ObjectId, Vec3A, f32)> = allies
                    .iter()
                    .filter(|(_, position, _)| {
                        threats.iter().any(|danger| danger.catches(*position, 0.0))
                    })
                    .collect();
                let classjob = spawn.common.class_job;
                let level = spawn.common.level;
                let ready: Vec<(u32, dodge::Brace, f32)> = game_data
                    .lock()
                    .brace_actions(classjob, level)
                    .into_iter()
                    .filter(|(id, ..)| bot.brace_ready.get(id).is_none_or(|at| *at <= now))
                    .collect();
                let kinds: Vec<(u32, dodge::Brace)> =
                    ready.iter().map(|(id, brace, _)| (*id, *brace)).collect();
                if let Some((action, brace)) = dodge::choose_brace(&kinds, caught.len(), true) {
                    let caster = threats
                        .iter()
                        .min_by_key(|danger| danger.until)
                        .map(|danger| danger.caster_id);
                    let target = match brace {
                        dodge::Brace::OnCaster => caster,
                        dodge::Brace::OnAlly => caught
                            .iter()
                            .min_by(|a, b| a.2.total_cmp(&b.2))
                            .map(|(id, ..)| *id),
                        dodge::Brace::AroundSelf | dodge::Brace::OnSelf => Some(*bot_id),
                    };
                    let recast = ready
                        .iter()
                        .find(|(id, ..)| *id == action)
                        .map(|(.., recast)| *recast)
                        .unwrap_or(0.0);
                    if let Some(target) = target {
                        kit_actions.push((*bot_id, action, target));
                        braces.push((*bot_id, until, action, recast));
                    }
                }
            }
            let distance = spot.map(|to| here.distance(to)).unwrap_or(0.0);
            let entry = dodge.get_or_insert(dodge::Dodge {
                to: None,
                set_off: dodge::set_off_at(now, until, distance, NPC_RUN_SPEED, fastrand::f32()),
                until,
                holding: false,
            });
            entry.until = entry.until.max(until);
            entry.to = spot;
        }
        let mut dodging = false;
        if let Some(current) = dodge {
            if threats.is_empty() && authored.is_none() && now >= current.until + dodge::LINGER {
                // Over: back to fighting, from where it stands.
                dodge = None;
                navmesh_path.clear();
                *navmesh_path_lerp = 0.0;
                *last_position = None;
            } else if now >= current.set_off {
                dodging = true;
                let flat = |a: Vec3A, b: Vec3A| Vec3A::new(a.x - b.x, 0.0, a.z - b.z).length();
                if let Some(to) = current.to
                    && flat(here, to) > AT_SPOT
                {
                    // Running there (a player would let a cast go, or slide it.)
                    let mut path = bot.path.clone();
                    let mut reaimed = bot.reaimed;
                    if path.back().is_none_or(|end| flat(*end, to) > AT_SPOT) {
                        // Along the navmesh only, never straight off it (Bob, 2026-10-24.)
                        path = instance_navmesh_path(navmesh, here, to).into();
                        reaimed = now;
                    }
                    let walled_for = bot
                        .walled_since
                        .map(|since| now.duration_since(since))
                        .unwrap_or_default();
                    let (moved, blocked) =
                        run_along(navmesh, collision.as_deref(), here, &mut path, walled_for);
                    walled.push((*bot_id, blocked.then(|| bot.walled_since.unwrap_or(now))));
                    if let Some((at, facing)) = moved {
                        spawn.common.position = Position(at);
                        spawn.common.rotation = facing;
                    }
                    walks.push((*bot_id, path, moved, reaimed));
                    dodges.push((
                        *bot_id,
                        Some(dodge::Dodge {
                            holding: false,
                            ..current
                        }),
                    ));
                    continue;
                }
                // Out of it (or nowhere better to be): it stays put and keeps acting below.
                walks.push((*bot_id, Default::default(), None, bot.reaimed));
                dodge = Some(dodge::Dodge {
                    holding: true,
                    ..current
                });
            }
        }
        dodges.push((*bot_id, dodge));
        // Out of combat it walks itself (below), so it stands still here while it casts (a Raise.)
        if casting && engaged.is_empty() {
            walks.push((*bot_id, bot.path.clone(), None, bot.reaimed));
            continue;
        }

        if engaged.is_empty() {
            if let Some(state) = &bot.party_npc {
                let mut state = state.clone();
                state.out_of_combat(now);
                npc_states.push((*bot_id, state));
            }
            // Out of combat: its spot in the diamond around the player.
            enmity.clear();
            if *state != NpcState::Follow {
                *state = NpcState::Follow;
                navmesh_path.clear();
                *navmesh_path_lerp = 0.0;
            }
            spawn.common.target_id = ObjectTypeId::default();
            *placement = Placement::default();
            let Some((anchor, _)) = where_is.get(&leader).copied() else {
                continue;
            };
            let facing = formation_facing.unwrap_or_default();
            let (ahead, side) = spots.get(bot_id).copied().unwrap_or((-2.0, 0.0));
            let spot = spot_around(anchor, facing, ahead, side);
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
                walled.push((*bot_id, None));
                far.push((*bot_id, None));
                continue;
            }
            // Too far behind to walk after them (a lift, a drop it won't take): beside them, after a moment.
            let far_since =
                (here.distance(anchor) > CATCH_UP_DISTANCE).then(|| bot.far_since.unwrap_or(now));
            if far_since.is_some_and(|since| now.duration_since(since) >= CATCH_UP_AFTER) {
                let to = match collision.as_deref() {
                    Some(collision) => collision.settle(spot),
                    None => instance.navmesh.ground_under(spot).unwrap_or(spot),
                };
                // On the player's floor: the spot's settled floor if it's near theirs, else where they stand.
                let to = if (to.y - anchor.y).abs() <= 2.0 {
                    to
                } else {
                    anchor
                };
                tracing::info!(
                    "Party bot {bot_id}: {:.0} yalms behind {leader} out of combat for {:.1} s; put beside them at {to}",
                    here.distance(anchor),
                    now.duration_since(far_since.unwrap_or(now)).as_secs_f32()
                );
                spawn.common.position = Position(to);
                spawn.common.rotation = facing;
                catch_ups.push((*bot_id, to, facing));
                walks.push((*bot_id, Default::default(), None, now));
                walled.push((*bot_id, None));
                far.push((*bot_id, None));
                continue;
            }
            far.push((*bot_id, far_since));
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
            let facing = f32::atan2(at.x - here.x, at.z - here.z);
            // On the floor, as a player walks (a straight stretch where the navmesh stops went through slopes,
            // through the air off a ledge, and up cliffs: Bob's Copperbell Mines bundle of 2026-10-26.)
            let walled_for = bot
                .walled_since
                .map(|since| now.duration_since(since))
                .unwrap_or_default();
            let blocked;
            (at, blocked) = walk_like_a_player(
                &instance.navmesh,
                collision.as_deref(),
                here,
                at,
                walled_for,
            );
            if !blocked
                && bot
                    .walled_since
                    .is_some_and(|since| now.duration_since(since) >= WALL_PATIENCE)
            {
                tracing::info!(
                    "Party bot {bot_id}: the zone's collision held it up at {here} on its way to {spot} for {:.1} s; on along the navmesh",
                    walled_for.as_secs_f32()
                );
            }
            walled.push((*bot_id, blocked.then(|| bot.walled_since.unwrap_or(now))));
            if flat(at, here) >= 0.05 {
                spawn.common.position = Position(at);
                spawn.common.rotation = facing;
                walks.push((*bot_id, path, Some((at, facing)), reaimed));
            } else {
                walks.push((*bot_id, path, None, reaimed));
            }
            continue;
        }
        if bot.walking_itself && !dodging {
            fighting.push(*bot_id);
        }

        // Whom it sees from where it stands (`npc_behavior::in_sight`): it acts on no one else, as a player can't.
        let sees = |position: Vec3A| {
            crate::server::npc_behavior::in_sight(
                collision.as_deref(),
                spawn.common.position.0,
                position,
            )
        };
        let sees_actor = |id: ObjectId| {
            where_is
                .get(&id)
                .map(|(position, _)| *position)
                .or_else(|| {
                    engaged
                        .iter()
                        .find(|enemy| enemy.id == id)
                        .map(|enemy| enemy.position)
                })
                .is_none_or(sees)
        };

        // The healer heals first.
        if bot.role == BotRole::Healer
            && let (Some(heal), Some((patient, _))) = (
                bot.heal_action,
                hurt.filter(|(_, percent)| *percent < bot.heal_below),
            )
            && now >= bot.next_heal
            && !casting
            && sees_actor(patient)
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
                        let delay = TANK_REACTIONS[fastrand::usize(..TANK_REACTIONS.len())]
                            * TANK_REACTION_SCALE;
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
                        && sees_actor(target)
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
        // Its fighting distance is from its target's edge, not its middle: against Ifrit (hitbox 3.5) retail's Duty
        // Support members fought from 6.5 (the Marauder, 3.0 in Sastasha), 7.6 (the Lancer, 3.4) and 13-15 yalms (the
        // Thaumaturge, 8.1) of his middle (Bob's capture b8432613, the Bowl of Embers); Kawari's stood inside him.
        let target_hitbox = engaged
            .iter()
            .find(|enemy| enemy.id == target)
            .map(|enemy| enemy.hitbox)
            .unwrap_or(0.0);
        timeline.attack_range = bot.attack_range + target_hitbox;
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
        } else if let Some(tank) = tank.filter(|tank| tank != bot_id) {
            // A melee damage dealer fights from behind what the tank holds, out of its frontal cones: retail's Duty
            // Support Lancer was caught by Ifrit's Incinerate (a cone on the tank) only after the tank had fallen, where
            // Kawari's stood in front and took 15 of them (Bob's bundles of 2026-10-28.)
            Placement {
                behind: Some((tank, target_hitbox + MELEE_BEHIND)),
                ..Default::default()
            }
        } else {
            Placement::default()
        };

        // A party NPC fights with its own kit (`party_npc::Kit`), and talks now and then.
        if let Some(state) = &bot.party_npc {
            let mut state = state.clone();
            let here = spawn.common.position.0;
            let distance = engaged
                .iter()
                .find(|enemy| enemy.id == target)
                .map(|enemy| {
                    Vec3A::new(enemy.position.x - here.x, 0.0, enemy.position.z - here.z).length()
                })
                .unwrap_or(f32::INFINITY);
            let special = specials.iter().any(|(id, ..)| id == bot_id);
            if special {
                state.used_gcd(now);
            } else if !casting {
                let provoking = provokes
                    .iter()
                    .find(|(id, _)| id == bot_id)
                    .map(|(_, enemy)| *enemy);
                if let Some(enemy) = provoking
                    && sees_actor(enemy)
                    && let Some(provoke) = state.provoke(now)
                {
                    kit_actions.push((*bot_id, provoke, enemy));
                } else if let Some(defensive) = state.defensive(now, engaged.len()) {
                    kit_actions.push((*bot_id, defensive, *bot_id));
                } else if let Some((action, on_self)) = {
                    let in_sight = sees_actor(target);
                    state.choose(
                        now,
                        in_sight && distance <= bot.attack_range + target_hitbox + PARTY_NPC_REACH,
                        in_sight && distance <= TANK_PULL_RANGE,
                    )
                } {
                    kit_actions.push((*bot_id, action, if on_self { *bot_id } else { target }));
                }
            }
            if let Some(line) = state.chatter(now) {
                lines.push((*bot_id, line));
            }
            npc_states.push((*bot_id, state));
        }
    }

    for (bot_id, dodge) in dodges {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.dodge = dodge;
        }
    }
    for (bot_id, until, action, recast) in braces {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.braced_for = Some(until);
            bot.brace_ready
                .insert(action, now + Duration::from_secs_f32(recast.max(BOT_PACE)));
        }
    }
    for (bot_id, state) in npc_states {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.party_npc = Some(state);
        }
    }
    for (bot_id, action, target) in kit_actions {
        npc_use_action(network, game_data, instance, bot_id, action, target);
    }
    for (bot_id, line) in lines {
        party_npc::yell(network, instance, bot_id, line);
    }

    for (bot_id, since) in walled {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.walled_since = since;
        }
    }
    for (bot_id, since) in far {
        if let Some(bot) = instance.party_bots.get_mut(&bot_id) {
            bot.far_since = since;
        }
    }
    // Put there at once (ActorSetPos, as the director's `warp_bnpc`), with the warp-in effect a zoning player shows.
    for (bot_id, position, rotation) in catch_ups {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(
            kawari::ipc::zone::ActorSetPos {
                position: Position(position),
                rotation,
                ..Default::default()
            },
        ));
        let mut network = network.lock();
        network.send_to_instance(
            ObjectId::default(),
            instance,
            FromServer::PacketSegment(ipc, bot_id),
            DestinationNetwork::ZoneClients,
        );
        network.send_ac_in_range_inclusive_instance(
            instance,
            bot_id,
            ActorControlCategory::ZoneIn {
                warp_finish_anim: 1,
                raise_anim: 0,
                unk1: 0,
            },
        );
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
        // Its GCD goes on the heal, so its rotation doesn't cast over it.
        let mechanics = game_data.lock().get_action_mechanics(heal);
        if let Some(NetworkedActor::Npc {
            rotation_state,
            timeline,
            ..
        }) = instance.actors.get_mut(&bot_id)
            && let Some(rotation) = &timeline.rotation
        {
            rotation_state.record(rotation, heal, patient, now, mechanics);
        }
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

    #[test]
    fn every_race_the_creator_offers() {
        // Two rows as CharaMakeType has them: a Hrothgar woman and a Viera man, with their own lists.
        let creator = [
            CreatorLook {
                race: Race::Hrothgar as u8,
                tribe: Tribe::Hellion as u8,
                gender: Gender::Female as u8,
                faces: vec![1, 2, 3, 4, 5],
                hairstyles: vec![201, 202],
                race_features: vec![1, 2, 3, 4],
            },
            CreatorLook {
                race: Race::Viera as u8,
                tribe: Tribe::Veena as u8,
                gender: Gender::Male as u8,
                faces: vec![1, 2, 3, 4],
                hairstyles: vec![],
                race_features: vec![1, 2],
            },
        ];
        let mut seen = HashSet::new();
        for _ in 0..200 {
            let look = look_from(&creator);
            seen.insert((look.race as u8, look.gender as u8));
            match look.race {
                Race::Hrothgar => {
                    assert_eq!((look.tribe, look.gender), (Tribe::Hellion, Gender::Female));
                    assert!((1..=5).contains(&look.face));
                    assert!([201, 202].contains(&look.hair));
                }
                Race::Viera => {
                    assert_eq!((look.tribe, look.gender), (Tribe::Veena, Gender::Male));
                    // No hairstyle list: the safe 1-4.
                    assert!((1..=4).contains(&look.hair));
                    assert!((1..=2).contains(&look.race_feature_type));
                }
                other => panic!("{other:?} isn't in the creator's list"),
            }
        }
        assert_eq!(seen.len(), 2);
        // Without the creator's data: the first six races.
        let look = look_from(&[]);
        assert!((1..=6).contains(&(look.race as u8)));
    }

    #[test]
    fn a_kept_friends_look_comes_back_the_same() {
        let look = CustomizeData {
            race: Race::Viera,
            tribe: Tribe::Rava,
            gender: Gender::Male,
            hair: 7,
            ..Default::default()
        };
        let text = look_to_hex(&look);
        assert_eq!(text.len(), 52);
        assert_eq!(
            look_from_hex(&text).map(|back| look_to_hex(&back)),
            Some(text)
        );
        assert!(look_from_hex("zz").is_none());
    }

    #[test]
    fn examine_laid_out_as_retail() {
        let common = CommonSpawn {
            class_job: 38,
            level: 60,
            glasses_ids: [460, 0],
            max_health_points: 8863,
            max_resource_points: 10000,
            second_model_stain_ids: [0, 0, 0, 0x66, 6, 0, 0, 0, 0, 0],
            ..Default::default()
        };
        let mut equipped = EquippedStorage::default();
        equipped.main_hand.item_id = 27358;
        equipped.head = Item {
            item_id: 49700,
            glamour_id: 14841,
            stains: [6, 0x5f],
            quantity: 1,
            ..Default::default()
        };
        equipped.soul_crystal.item_id = 25217;
        let stats = BaseParameters {
            strength: 199,
            dexterity: 800,
            haste: 100,
            ..Default::default()
        };
        let (first, second) = examine_bytes(&common, 886, Some(&equipped), 201, Some(&stats));
        // The whole packet is these, the name (32 bytes) between: 640 + 32 + 272 = 944, as retail's.
        assert_eq!(first.len() + 32 + second.len(), 944);
        assert_eq!(&first[0..4], &[4, 1, 38, 60]);
        assert_eq!(u16::from_le_bytes([first[6], first[7]]), 886);
        assert_eq!(u16::from_le_bytes([first[72], first[73]]), 201);
        let u32_at =
            |bytes: &[u8], at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        // Slots of 40 bytes from 80: the main hand first, the head third, the soul crystal last.
        assert_eq!(u32_at(&first, 80), 27358);
        assert_eq!(u32_at(&first, 80 + 2 * 40), 49700);
        assert_eq!(u32_at(&first, 80 + 2 * 40 + 4), 14841);
        assert_eq!(&first[80 + 2 * 40 + 38..80 + 2 * 40 + 40], &[6, 0x5f]);
        assert_eq!(u32_at(&first, 80 + 13 * 40), 25217);
        // After the name: the dyes at 772 and facewear at 782 of the packet; the stats from 816.
        assert_eq!(second[100 + 3], 0x66);
        assert_eq!(u16::from_le_bytes([second[110], second[111]]), 460);
        assert_eq!(u32_at(&second, 816 - 672), 199);
        assert_eq!(u32_at(&second, 816 - 672 + 4), 800);
        assert_eq!(u32_at(&second, 816 - 672 + 6 * 4), 8863);
        assert_eq!(u32_at(&second, 816 - 672 + 25 * 4), 100);
        // A player examined has no gear or stats on the server's side: those stay 0.
        let (first, second) = examine_bytes(&common, 0, None, 0, None);
        assert_eq!(u32_at(&first, 80), 0);
        assert_eq!(u32_at(&second, 816 - 672), 0);
        assert_eq!(
            bot_content_id(&Instance::default(), ObjectId(0x1234_5678)),
            0x00B0_0000_1234_5678
        );
    }

    #[test]
    fn a_bot_shown_as_a_player() {
        let npc = SpawnNpc {
            common: CommonSpawn {
                object_kind: ObjectKind::BattleNpc(BattleNpcSubKind::Chocobo),
                owner_id: ObjectId(0x1000_0001),
                name: "Rana Velis".to_string(),
                level: 18,
                class_job: 1,
                health_points: 400,
                max_health_points: 450,
                battalion: 0,
                ..Default::default()
            },
            character_data_flags: CharacterDataFlag::from_bits_retain(0x1),
            ..Default::default()
        };
        let shown = ShownAsPlayer {
            account_id: BOT_ID_BASE | 0x1234_5678,
            content_id: BOT_ID_BASE | 0x1234_5678,
            world_id: 40,
            handler_id: HandlerId(0x8003_0004),
        };
        let spawn = player_spawn(&npc, &shown);
        assert_eq!(
            spawn.common.object_kind,
            ObjectKind::Player(PlayerSubKind::Player)
        );
        assert_eq!(spawn.common.owner_id, ObjectId::default());
        assert_eq!(spawn.common.display_flags, DisplayFlag::UNK1);
        assert_eq!(spawn.common.handler_id, HandlerId(0x8003_0004));
        assert_eq!(spawn.content_id, BOT_ID_BASE | 0x1234_5678);
        assert_eq!((spawn.current_world_id, spawn.home_world_id), (40, 40));
        assert_eq!(spawn.online_status, OnlineStatus::PartyMember);
        // What the client sees of it is the NPC's as it is now.
        assert_eq!(spawn.common.name, "Rana Velis");
        assert_eq!(
            (spawn.common.health_points, spawn.common.max_health_points),
            (400, 450)
        );
        assert_eq!((spawn.common.level, spawn.common.class_job), (18, 1));
        for word in ["player", "chocobo", "partymember"] {
            assert_eq!(BotLook::from_word(word).map(BotLook::word), Some(word));
        }
        // A hat it can't show is hidden, as with the headgear button.
        let mut hatless = npc.clone();
        hatless.common.display_flags = DisplayFlag::HIDE_HEAD;
        assert_eq!(
            player_spawn(&hatless, &shown).common.display_flags,
            DisplayFlag::UNK1 | DisplayFlag::HIDE_HEAD
        );
    }

    #[test]
    fn race_and_sex_gear() {
        // A Viera woman's wearable rows, say: everyone's (1), women's (3) and Viera women's own (19).
        let wearable = HashSet::from([1, 3, 19]);
        assert!(can_wear(&wearable, 0));
        assert!(can_wear(&wearable, 1));
        assert!(can_wear(&wearable, 19));
        // Men's, or another race's own (a Miqo'te loincloth): left out.
        assert!(!can_wear(&wearable, 2));
        assert!(!can_wear(&wearable, 11));
        // Without the sheet, anything (as before.)
        assert!(can_wear(&HashSet::new(), 11));
    }

    #[test]
    fn a_healer_finds_its_raise() {
        let cure = ClassAction {
            heals: true,
            hostile: false,
            ..action(120, 2, true, 0)
        };
        let raise = ClassAction {
            hostile: false,
            range: 30,
            ..action(125, 12, true, 0)
        };
        assert_eq!(raise_action(&[cure, raise]), Some((125, 30.0)));
        assert_eq!(raise_action(&[cure]), None);
    }

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
            recast: 2.5,
            mp_cost: 0,
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

        // Afflatus Solace needs a Lily: the higher level heal isn't the bot's.
        let mut solace = action(16531, 52, true, 0);
        solace.hostile = false;
        solace.heals = true;
        assert_eq!(heal_action(&[heal, solace]), Some(120));
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
    fn the_diamond_as_retails_duty_support() {
        // Bob's capture: the player a healer; the tank 2 yalms ahead, the Lancer and Thaumaturge 1 ahead, 1.72 aside.
        let (tank, lancer, thaum) = (ObjectId(1), ObjectId(2), ObjectId(3));
        let spots = diamond_spots(
            ROLE_HEALER,
            &[
                (tank, ROLE_TANK),
                (thaum, ROLE_RANGED),
                (lancer, ROLE_MELEE),
            ],
        );
        assert_eq!(spots[&tank], (2.0, 0.0));
        assert_eq!(spots[&lancer], (1.0, 1.72));
        assert_eq!(spots[&thaum], (1.0, -1.72));

        // The player a tank: everyone behind them, the healer at the back.
        let healer = ObjectId(4);
        let spots = diamond_spots(
            ROLE_TANK,
            &[
                (healer, ROLE_HEALER),
                (lancer, ROLE_MELEE),
                (thaum, ROLE_RANGED),
            ],
        );
        assert_eq!(spots[&healer], (-2.0, 0.0));
        assert_eq!(spots[&lancer], (-1.0, 1.72));
        assert_eq!(spots[&thaum], (-1.0, -1.72));

        // Two melee bots for a ranged player: the second takes the other side; more than the diamond stand behind.
        let bots: Vec<(ObjectId, u8)> = (10..16).map(|i| (ObjectId(i), ROLE_MELEE)).collect();
        let spots = diamond_spots(ROLE_RANGED, &bots);
        assert_eq!(spots[&ObjectId(10)], (0.0, 3.44));
        let all: Vec<(f32, f32)> = bots.iter().map(|(id, _)| spots[id]).collect();
        for (i, a) in all.iter().enumerate() {
            assert!(a.0.hypot(a.1) >= 1.5, "spot {i} is on the player");
            for (j, b) in all.iter().enumerate().skip(i + 1) {
                assert!(
                    (a.0 - b.0).hypot(a.1 - b.1) >= 1.0,
                    "spots {i} and {j} overlap"
                );
            }
        }
    }

    #[test]
    fn spots_turn_with_the_player() {
        let here = Vec3A::new(10.0, 5.0, -3.0);
        // Facing +Z: ahead is +Z, the melee side +X.
        let spot = spot_around(here, 0.0, 2.0, 1.0);
        assert!((spot - Vec3A::new(11.0, 5.0, -1.0)).length() < 1e-5);
        // Facing +X (a quarter turn): ahead is +X, the melee side -Z.
        let spot = spot_around(here, std::f32::consts::FRAC_PI_2, 2.0, 1.0);
        assert!((spot - Vec3A::new(12.0, 5.0, -4.0)).length() < 1e-5);
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
