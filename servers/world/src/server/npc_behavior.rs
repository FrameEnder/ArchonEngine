use std::{
    collections::{HashMap, HashSet, VecDeque},
    f32::consts::PI,
    sync::Arc,
    time::{Duration, Instant},
};

use glam::Vec3A;
use kawari::{
    common::{
        AUTO_ATTACK_RATE, JumpState, MOB_WANDER_JITTER, MOB_WANDER_TIME, MoveAnimationState,
        MoveAnimationType, ObjectId, ObjectTypeId, ObjectTypeKind, Position,
        STRIKING_DUMMY_NAME_ID, TimepointData, TimepointFacing, TimepointTarget,
    },
    ipc::zone::{
        ActionRequest, ActionType, ActorControlCategory, ActorSetPos, BattleNpcSubKind,
        CharacterDataFlag, ObjectKind, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer, GameData,
    navmesh::MAX_OFF_MESH_GAP,
    server::{
        actor::{NetworkedActor, NpcCast, NpcState, NpcTarget, set_shared_group_timeline_state},
        first_attack,
        instance::{Instance, QueuedTaskData},
        modifiers,
        network::{DestinationNetwork, NetworkState},
        party_bot::KnownAction,
        rotation::{COMBO_WINDOW, RotationState, Situation},
    },
    status_effects::StatusEffects,
};

/// How far around a target (or a bot) its area attacks count enemies, in yalms: the usual 5-yalm circle.
const BOT_AREA_REACH: f32 = 5.0;
/// A GCD action's recast when the sheet doesn't say (2.5 s.)
const BOT_GCD: f32 = 2.5;

/// What a party bot with a rotation script sees around it (`bot_choice`), worked out before NPCs act.
struct BotSurroundings {
    target_statuses: Vec<(u16, f32, bool)>,
    near_target: u32,
    near_self: u32,
    distance: f32,
    gauge: Vec<(&'static str, i32)>,
    known: Vec<KnownAction>,
}

/// What a party bot's rotation script (`bot_rotation`) picks now, if it can be used now: an action it knows, off
/// cooldown, a GCD action only when its GCD is back and an ability only between them; on its target, or on itself
/// for one used on itself.
fn bot_choice(
    id: ObjectId,
    spawn: &kawari::ipc::zone::SpawnNpc,
    statuses: &StatusEffects,
    state: &RotationState,
    around: &BotSurroundings,
    situation: &Situation,
    target: ObjectId,
) -> Option<(u32, ObjectId)> {
    let now = situation.now;
    let since = |at: Instant| now.saturating_duration_since(at).as_secs_f32();
    let last_gcd = state.last_gcd();
    let recast_of = |action: u32| {
        around
            .known
            .iter()
            .find(|known| known.id == action)
            .map(|known| known.recast)
            .filter(|recast| *recast > 0.0)
            .unwrap_or(BOT_GCD)
    };
    let (gcd_left, since_gcd) = match last_gcd {
        Some((action, at)) => ((recast_of(action) - since(at)).max(0.0), since(at)),
        None => (0.0, 99.0),
    };
    // The combo goes by its combo steps (`RotationState::last_combo`): the GCD actions outside combos between leave it be.
    let combo = state
        .last_combo()
        .filter(|(_, at)| now.saturating_duration_since(*at) <= COMBO_WINDOW)
        .map(|(action, _)| action)
        .unwrap_or(0);
    let known: Vec<(u32, bool, f32, u32)> = around
        .known
        .iter()
        .map(|known| {
            // A GCD action's own recast is the GCD's (`gcd_left`); one with a longer recast of its own counts it.
            let left = state
                .used_at(known.id)
                .filter(|_| !known.gcd || known.recast > BOT_GCD)
                .map(|at| (known.recast - since(at)).max(0.0))
                .unwrap_or(0.0);
            (known.id, known.gcd, left, known.mp_cost)
        })
        .collect();
    let own_statuses = statuses
        .data()
        .iter()
        .map(|effect| {
            let left = statuses
                .time_left(effect.effect_id)
                .map(|left| if left <= 0.0 { 999.0 } else { left })
                .unwrap_or(0.0);
            (effect.effect_id, left, effect.param)
        })
        .collect();
    let view = crate::server::bot_rotation::BotView {
        bot_id: id.0,
        classjob: spawn.common.class_job,
        level: spawn.common.level,
        mp: spawn.common.resource_points as u32,
        max_mp: spawn.common.max_resource_points as u32,
        hp: situation.self_hp,
        target_hp: situation.target_hp,
        gcd_left,
        since_gcd,
        weaves: state.weaves(),
        combo,
        known,
        statuses: own_statuses,
        target_statuses: around.target_statuses.clone(),
        near_target: around.near_target,
        near_self: around.near_self,
        distance: around.distance,
        gauge: around.gauge.clone(),
        in_combat: 0.0,
    };
    let (action, on_self) = crate::server::bot_rotation::choose(&view)?;
    let known = around.known.iter().find(|known| known.id == action)?;
    let left = view
        .known
        .iter()
        .find(|(id, _, _, _)| *id == action)
        .map(|(_, _, left, _)| *left)
        .unwrap_or(0.0);
    let timely = if known.gcd {
        gcd_left <= 0.25 && left <= 0.25
    } else {
        left <= 0.25
    };
    // Its MP has to cover the cost, but for the Thaumaturge's and Black Mage's (their gauge changes it: their script
    // checks.)
    let gauge_costs = matches!(spawn.common.class_job, 7 | 25);
    if !timely || (!gauge_costs && known.mp_cost > spawn.common.resource_points as u32) {
        return None;
    }
    let target = if on_self || !known.hostile {
        id
    } else {
        target
    };
    Some((action, target))
}

/// How often NPCs auto-attack, in half-second ticks: every 3 seconds, which is what Kawari Bestiary recorded enemies
/// doing on retail (their hits landed 3.0-3.1 seconds apart.)
const NPC_AUTO_ATTACK_TICKS: i64 = AUTO_ATTACK_RATE * 2;

/// How long an NPC that lost its target waits before sheathing its weapon.
const NPC_SHEATHE_DELAY: Duration = Duration::from_secs(2);

/// How close (in yalms) a player has to come for an aggressive enemy to notice them: within its sight, a ±45° cone in
/// front of it, up to 15 yalms; all round within `AGGRO_PROXIMITY`. From Bob's Chronofoil captures (98 enemies that
/// came at him on their own: 2-18 yalms, median 9; every one from behind was within about 11.)
const AGGRO_RANGE: f32 = 15.0;
/// How close an aggressive enemy notices players whichever way it faces.
const AGGRO_PROXIMITY: f32 = 11.0;
/// Half the width of an aggressive enemy's sight, in radians (45°.)
const AGGRO_SIGHT_HALF_ANGLE: f32 = PI / 4.0;

/// Aggressive enemies ignore players this many levels or more above them: on retail they still came at Bob 7 and 8
/// levels above them, never 9 or more (195 close passes.)
const AGGRO_LEVEL_GAP: u8 = 9;

/// How fast enemies move, in yalms a second: walking about while idle, and running (chasing, going home.) Retail's
/// ActorMove speed byte is this in tenths (20 and 60), Bob's Chronofoil captures.
const NPC_WALK_SPEED: f32 = 2.0;
pub const NPC_RUN_SPEED: f32 = 6.0;
/// An enemy's cast bar is this much shorter than the action's cast time, which it still lands at: retail's ActorCast
/// carried the Action sheet's time minus 0.3 s for 99 % of 8,392 enemy casts (Bob's Chronofoil captures.)
pub const NPC_CAST_BAR_SHORTER_BY: f32 = 0.3;

/// The cast time an NPC's ActorCast shows for an action taking `cast_time` seconds.
pub fn npc_cast_bar_time(cast_time: f32) -> f32 {
    if cast_time > NPC_CAST_BAR_SHORTER_BY {
        cast_time - NPC_CAST_BAR_SHORTER_BY
    } else {
        cast_time
    }
}

/// How much of a second one behaviour tick is (`SERVER_TICK_MILLISECONDS`.)
pub const BEHAVIOR_TICK_SECONDS: f32 = 0.5;

/// How far (in yalms) an enemy will chase from where it spawned before giving up, going home and healing up.
/// TODO: per enemy, from Kawari Recorder's "World" recording (gave up at ~N yalms from spawn.)
const LEASH_DISTANCE: f32 = 40.0;
/// How many behaviour ticks (half a second each) an NPC waits at a wall (or a drop) before walking on along its path,
/// as party bots do (`party_bot::WALL_PATIENCE`, 2 s). Yda stood 16 yalms from the beast she was sent after for the rest
/// of Brotherly Love's battle (Bob's bundles 20261030-110117 and -110334): her path ended where the navmesh did, the
/// rest went straight, and the collision stopped every step of it; with the path never used up she never planned
/// another. Retail's NPCs aren't held up like that: its server walks them on the navmesh alone. Open-world enemies too
/// (a Ziz in the North Shroud stood 8.5 yalms short of the player it was after, never attacking, for as long as he
/// watched: Bob's bundle 20261101-215226). Out there a path keeps to the navmesh but for at most its last
/// `MAX_OFF_MESH_GAP` yalms (where there's no mesh to the one it's after), so walking on keeps them to where the
/// navmesh says they can go, not through the world's walls or off its cliffs.
const NPC_WALL_PATIENCE_TICKS: u32 = 4;

/// How long (in behaviour ticks) an enemy goes without reaching the one it's after, who's beyond `LEASH_DISTANCE` of
/// its home, before giving up: 7 s, the shortest chase before retail's enemies gave up (Bob's Chronofoil captures.)
const GIVE_UP_UNREACHED_TICKS: i64 = 14;

/// How close to home an enemy that gave up has to get before it pays attention again.
const HOME_DISTANCE: f32 = 3.0;
/// An NPC this close (yalms, across the ground) to the spot it walks to has got there.
const ARRIVE_DISTANCE: f32 = 0.5;
/// Path points closer than this (yalms, across the ground) to where an NPC stands are where it already is.
const MIN_STEP: f32 = 0.05;
/// A path that ends less than this much nearer (yalms, across the ground) to an NPC's spot than where it stands gets it
/// no closer: it has arrived.
const NO_CLOSER_MARGIN: f32 = 0.1;
/// How long an open-world NPC leaves alone someone it couldn't get to (no navmesh path, or one that gets it no closer):
/// the Brass Blade at Scorpion Crossing kept noticing a monster it couldn't reach, walked back to its post, noticed it
/// again, and stepped between its post (off the navmesh) and the mesh's edge every half second (Bob's bundle of
/// 2026-10-25.)
const UNREACHABLE_FOR: Duration = Duration::from_secs(60);

/// The distance between two points across the ground: the navmesh's heights are a little off the NPCs' (a quarter
/// yalm on a flat test mesh), so a path to where it stands starts that far above it.
fn flat_distance(a: Vec3A, b: Vec3A) -> f32 {
    Vec3A::new(a.x - b.x, 0.0, a.z - b.z).length()
}
/// How far an open-world enemy steps down off a ledge (yalms; a guess): it doesn't throw itself off a cliff after
/// someone, as retail's keep to their navmesh. Bots, pets and everyone in duties go down as far as a player would.
const OPEN_WORLD_MAX_DROP: f32 = 3.0;

/// How far below its path a walker that goes anywhere (bots, pets, NPCs in duties) may step down: as far as its path
/// goes down, and this much more (a guess, the open world's.) Off the path's height it stopped there instead of
/// dropping, so it no longer fell where the collision has no floor under its way: Bob's Haukke Manor bundle of
/// 2026-11-02, where the tank bot and three enemies, cutting a corner of their path at (-32, 0, 45), stepped into the
/// sunken space beside it (floor at -3.2, no railing in Kawari's collision) and on down to the cellar 18.8 yalms under
/// the player, which on retail no one ever reached (four runs.)
pub const PATH_DROP_MARGIN: f32 = 3.0;

/// The deepest drop a walker going anywhere takes, stepping from `here` towards a point of its path at `along`.
pub fn path_max_drop(here: Vec3A, along: Vec3A) -> f32 {
    (here.y - along.y).max(0.0) + PATH_DROP_MARGIN
}

/// Where a walker held up by a wall at `here` goes on to, its patience run out, towards `along` (a point of its path):
/// there, on the floor no higher than a step over the higher of the two and no deeper than `max_drop` under `here`.
/// None where a gimmick the director put up stands between (`Collision::crosses_sealed`), or there's no such floor:
/// put on the path's height over a hole, it sank to the floor below the next tick; Bob's Haukke Manor bundle
/// 20261103-051833, where bots and enemies walked on past a shelf by the sunken space at (-32, 0, 45) and on down to
/// the cellar, as they did through the doors it shows shut.
pub fn past_a_wall(
    collision: &crate::collision::Collision,
    here: Vec3A,
    along: Vec3A,
    max_drop: f32,
) -> Option<Vec3A> {
    if collision.crosses_sealed(here, along) {
        return None;
    }
    let y = collision.floor(
        along.x,
        along.z,
        here.y.max(along.y) + collision.climb,
        here.y - max_drop,
    )?;
    Some(Vec3A::new(along.x, y, along.z))
}

/// Whether an NPC at `from` sees whom it's after at `to`, on its instance's `collision` (`Collision::sees`): through a
/// wall, a floor or a door the director shut, it doesn't act on them; it walks on until it does. Without a collision
/// (not made yet) it sees everyone. Bob's Haukke Manor bundles 20261103-051638 and -051833: bots and enemies attacked
/// through walls.
pub fn in_sight(collision: Option<&crate::collision::Collision>, from: Vec3A, to: Vec3A) -> bool {
    collision.is_none_or(|collision| collision.sees(from, to))
}
/// NPC actions that leap their user to its target, and how long after the hit it lands (seconds). Retail's Duty Support
/// Marauder opens a pull with 29291 from about 10 yalms off: the hit (a 1.1 s lock), then 0.58 s later ActorControl
/// 239 (`SnapNextMove`) and an ActorMove (speed 60) to beside its target, three times out of three in Bob's Copperbell
/// Mines capture (bd298651 visit 0.5, 2026-10-25). Kawari sent the hit and left him where he was, so the client played
/// the leap and put him back: "he jumps to the enemy and teleports back" (Bob, 2026-10-26.)
pub const NPC_LEAPS: [(u32, f32); 1] = [(29291, 0.58)];
/// How far short of its target's middle a leap lands (yalms; retail's Marauder landed 1.1, 2.1 and 2.4 short.)
const LEAP_SHORT_OF: f32 = 2.0;
/// The landing's ActorMove speed byte (retail's.)
const LEAP_MOVE_SPEED: u8 = 60;
/// How long after its cast ends an NPC keeps the facing it cast with (seconds): its hit goes out the tick after, and
/// turning back to its target in between swung Kottos' Catapult the tank's way (Bob's bundle of 2026-10-26.)
const KEEP_CAST_FACING: f32 = 0.6;
/// How many NPCs that can't wander an instance logs (the rest are only counted.)
const STUCK_WANDERERS_LOGGED: usize = 5;

/// A timeline action an NPC is walking into reach to use (`TimepointData::Action`'s `reach`): retail's Notorious Biggs
/// boss left the fighter it fought, walked 3.4-4.1 s to whoever its 1068 was for and cast it from 9.1-10.5 yalms (Bob's
/// Chronofoil captures 689b15da, da424662, 898dd0b9: four runs.) While it's walking it neither auto-attacks nor
/// uses the rest of its timeline, which comes once it's done, as after a cast.
#[derive(Debug, Clone)]
pub struct PendingAction {
    /// The timepoint's action.
    pub data: TimepointData,
    /// Whom it's for (chosen when it came due: `TimepointTarget::Other` picks once.)
    pub on: ObjectId,
    /// How near it has to be (yalms, centre to centre.)
    pub reach: f32,
    /// When it came due.
    pub since: Instant,
}

/// How long an NPC keeps walking to use a timeline action before it gives it up (whom it's for out of its way: rooted,
/// or always running off.) Retail's walks took 3.4-4.1 s.
const PENDING_ACTION_PATIENCE: Duration = Duration::from_secs(10);

/// How near (centre to centre) an NPC with this `attack_range` gets to a target with a hitbox of `target_hitbox`:
/// its range is measured to a player-sized target (as the timelines' were, from where retail's stopped), and a bigger
/// one is reached that much sooner, from its edge. Retail's Notorious Biggs escort stopped 3.0-3.7 yalms from the
/// small enemies and 5.5-5.7 from the big ones (1655, 1657), and the fighter and conjurer likewise (Bob's Chronofoil
/// captures 689b15da, da424662, 898dd0b9: four runs); Kawari's stood 3.5 from both.
pub fn reach_for_size(attack_range: f32, target_hitbox: Option<f32>) -> f32 {
    let player = crate::server::action::PLAYER_HITBOX_RADIUS;
    attack_range + target_hitbox.map_or(0.0, |radius| (radius - player).max(0.0))
}

/// An NPC at `here` walking to use a timeline action (`pending`): the action and whom it's for once they're within its
/// reach (it's no longer pending then); nothing while it's still on its way. It gives up (no longer pending, nothing to
/// use) when they're gone or it has walked for `PENDING_ACTION_PATIENCE`.
pub fn pending_action_ready(
    pending: &mut Option<PendingAction>,
    here: Vec3A,
    positions: &HashMap<ObjectId, Vec3A>,
) -> Option<(TimepointData, ObjectId)> {
    let waiting = pending.as_ref()?;
    let there = positions.get(&waiting.on).copied();
    if there.is_some_and(|at| Vec3A::distance(here, at) <= waiting.reach) {
        let done = pending.take()?;
        return Some((done.data, done.on));
    }
    if there.is_none() || waiting.since.elapsed() >= PENDING_ACTION_PATIENCE {
        *pending = None;
    }
    None
}

/// The range of an action the Action sheet gives as -1: its user's weapon's, a melee one's (yalms.)
const WEAPON_RANGE: f32 = 3.0;
/// How much further than an action's range, edge to edge, an NPC still uses it (yalms): where it stops isn't exact.
const ACTION_RANGE_SLACK: f32 = 1.0;

/// Whether a pull-relative timeline's action on someone else is left out because they're beyond the action's own
/// range (the Action sheet's, between the two hitboxes' edges as the client measures it; -1 is a weapon's reach, 0 an
/// action centred on its user, which has no such range) or out of sight. These actions were used on their second
/// "whether in reach or not": an enemy lancer that a wall kept 33 yalms from the reaper it was sent after used its
/// weaponskills (range 3) on it from there, facing the wall (Every Little Thing She Does Is Magitek, Bob's marks of
/// 2026-11-14: "Eques is attacking something but he's facing the wall?"); retail's used every one from 4.5 yalms
/// (three runs). A boss's spell on a tank who has stepped back still comes: its range is the spell's.
pub fn out_of_action_range(
    range: i8,
    distance: f32,
    own_hitbox: f32,
    target_hitbox: f32,
    in_sight: bool,
) -> bool {
    if range == 0 {
        return false;
    }
    let range = if range < 0 {
        WEAPON_RANGE
    } else {
        range as f32
    };
    !in_sight || distance > range + own_hitbox + target_hitbox + ACTION_RANGE_SLACK
}

/// Updates NPCs in this instance.
pub fn npc_behavior(
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    instance: &mut Instance,
    haters: &mut HashMap<ObjectId, Vec<ObjectId>>,
) {
    if instance.enemy_ai_disabled || !instance.navmesh.is_available() {
        return;
    }

    let mut actor_moves = Vec::new();
    let mut target_actor_pos = HashMap::new();
    let enemies = instance.find_possible_enemies();
    let untargetable = unnoticed(instance);
    let now = Instant::now();
    let paralysis_locked: HashSet<ObjectId> = instance
        .actors
        .keys()
        .filter(|id| instance.paralysis.locked(**id, now))
        .copied()
        .collect();
    // Players on a mount: no one attacks them until they get off. Retail's ambush in Bob's capture 1fac2416 (a quest's
    // EVENTRANGE ridden into, mounted): its enemies came out at once, and the first of them acted 2.4 s after he got off,
    // 5.3 s after they came; none touched him meanwhile. Kawari's hit the mounted player half a second after they came
    // (Bob's bundle 20261110-120801, By the Lights of Ishgard.)
    let mounted_players: HashSet<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| {
            matches!(actor, NetworkedActor::Player { .. })
                && actor.get_common_spawn().mode == kawari::common::CharacterMode::Mounted
        })
        .map(|(id, _)| *id)
        .collect();
    let levels: HashMap<ObjectId, u8> = instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, actor)| (*id, actor.get_common_spawn().level))
        .collect();
    // Everyone's health in %, for rotations that wait for a target (or themselves) to be low.
    let health_percent: HashMap<ObjectId, f32> = instance
        .actors
        .iter()
        .filter(|(_, actor)| {
            matches!(
                actor,
                NetworkedActor::Player { .. } | NetworkedActor::Npc { .. }
            )
        })
        .map(|(id, actor)| {
            let common = actor.get_common_spawn();
            let percent = if common.max_health_points == 0 {
                100.0
            } else {
                common.health_points as f32 * 100.0 / common.max_health_points as f32
            };
            (*id, percent)
        })
        .collect();
    // Each side's living battle NPCs by BNpcBase, one of each (the lowest layout ID), for rotation entries used
    // `on_ally` (the Akoman's drain on a yodomi, quest battle 17): (hostile, base) -> (layout, actor). The side is
    // the battalion's (at war with the players' or not, `action::at_war`), not the HOSTILE flag, which allies have too.
    let players_enemies = game_data
        .lock()
        .get_battalion_enemies(crate::server::action::PLAYER_BATTALION);
    let mut allies_by_base: HashMap<(bool, u32), (u32, ObjectId)> = HashMap::new();
    for (id, actor) in &instance.actors {
        if let NetworkedActor::Npc { spawn, .. } = actor
            && matches!(spawn.common.object_kind, ObjectKind::BattleNpc(_))
            && spawn.common.health_points > 0
        {
            let hostile = crate::server::action::at_war(&players_enemies, actor);
            let key = (hostile, spawn.common.base_id);
            let layout = spawn.common.layout_id;
            if allies_by_base
                .get(&key)
                .is_none_or(|(existing, _)| layout < *existing)
            {
                allies_by_base.insert(key, (layout, *id));
            }
        }
    }
    // Every battle NPC's hitbox bigger than a player's: one going for it stops that much further off
    // (`reach_for_size`.)
    let big_hitboxes: HashMap<ObjectId, f32> = instance
        .actors
        .iter()
        .filter_map(|(id, actor)| match actor {
            NetworkedActor::Npc { timeline, .. }
                if timeline.hitbox_radius > crate::server::action::PLAYER_HITBOX_RADIUS =>
            {
                Some((*id, timeline.hitbox_radius))
            }
            _ => None,
        })
        .collect();
    // Everyone's position, for NPCs told to stay near someone (the director's `keep_near`.)
    let positions: HashMap<ObjectId, Vec3A> = instance
        .actors
        .iter()
        .filter(|(_, actor)| {
            matches!(
                actor,
                NetworkedActor::Player { .. } | NetworkedActor::Npc { .. }
            )
        })
        .map(|(id, actor)| (*id, actor.get_common_spawn().position.0))
        .collect();
    // The players' positions, for an area an NPC aims at the nearest one (`TimepointFacing::Player`.)
    let player_positions: Vec<Vec3A> = instance
        .actors
        .values()
        .filter(|actor| {
            matches!(actor, NetworkedActor::Player { .. })
                && actor.get_common_spawn().health_points > 0
        })
        .map(|actor| actor.get_common_spawn().position.0)
        .collect();
    // Who everyone is targeting, for NPCs placed behind the tank's target.
    let targets: HashMap<ObjectId, ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| {
            matches!(
                actor,
                NetworkedActor::Player { .. } | NetworkedActor::Npc { .. }
            )
        })
        .map(|(id, actor)| (*id, actor.get_common_spawn().target_id.object_id))
        .filter(|(_, target)| target.is_valid())
        .collect();
    let mut gave_up = Vec::new();
    let in_content = instance.content_id != 0 || instance.content_finder_condition_id != 0;
    // In content an NPC that can't path to its target walks the rest of the way straight, however far: our navmesh has
    // holes the arenas don't, and an ally or add stuck at one stalls the whole battle (2026-10-16 bundle).
    let max_off_mesh_gap = if in_content {
        f32::INFINITY
    } else {
        MAX_OFF_MESH_GAP
    };

    let busy_now = Instant::now();
    instance.busy_npcs.retain(|_, until| *until > busy_now);
    instance.rooted_npcs.retain(|_, until| *until > busy_now);
    // NPCs whose step a wall or a drop cut short this tick: their path goes on from where they stopped.
    let mut walled: HashSet<ObjectId> = HashSet::new();
    // Every NPC a wall stopped this tick, whether or not it then walked on through it (`NPC_WALL_PATIENCE_TICKS`.)
    let mut blocked: HashSet<ObjectId> = HashSet::new();

    // const pass
    for (id, actor) in &instance.actors {
        // Kept busy by its director (`director:busy_bnpc`): it stays as it is. Kept where it stands
        // (`director:root_bnpc`): it doesn't move (its actions are the mut pass's.)
        if instance.busy_npcs.contains_key(id) || instance.rooted_npcs.contains_key(id) {
            continue;
        }
        // A director's helper does only what it's told (`director:hold_bnpc`); a world bot only what `world_bots.rs`
        // has it do.
        // Before the duty begins, everyone waits (`Instance::awaiting_commence`.)
        if instance.held_npcs.contains(id)
            || instance.world_bots.contains(id)
            || instance.awaiting_commence
        {
            continue;
        }
        // A party bot out of combat walks itself (server/party_bot.rs.) One waiting out an area attack still acts
        // (the mut pass), so where its target stands is noted here all the same: left out, its target had no place,
        // nothing was in its reach, and the party stood at their spots through the Stone Vigil's whirlwinds without
        // an attack or a spell for 24 s (Bob's bundle 20261110-182408: "bots don't attack during dodging"; retail's
        // Duty Support members and players alike kept on, 9-23 actions each in the same stretch.)
        if instance
            .party_bots
            .get(id)
            .is_some_and(|bot| bot.walking_itself)
        {
            if let NetworkedActor::Npc {
                navmesh_target: Some(NpcTarget::Actor(target)),
                ..
            } = actor
                && let Some(target_actor) = instance.find_actor(*target)
            {
                target_actor_pos.insert(*target, target_actor.get_common_spawn().position.0);
            }
            continue;
        }
        if let NetworkedActor::Npc {
            navmesh_path: current_path,
            navmesh_path_lerp: current_path_lerp,
            navmesh_target: current_target,
            spawn,
            last_position,
            state,
            timeline,
            cast,
            placement,
            status_effects,
            returning_home,
            ..
        } = actor
            && let Some(current_target) = current_target
            && !cast.is_some_and(|cast| {
                // (An auto-attack's swing doesn't hold it.)
                let keep = if cast.action_id == timeline.autoattack_action_id {
                    0.0
                } else {
                    KEEP_CAST_FACING
                };
                cast.until + Duration::from_secs_f32(keep) > Instant::now()
            })
            // Stunned, asleep or bound: it doesn't move.
            && !modifiers::is_rooted(status_effects)
            && *state != NpcState::Dead
            && *state != NpcState::AoeTest
            && spawn.common.health_points > 0
            && spawn.common.name_id != STRIKING_DUMMY_NAME_ID
        {
            let target_pos;
            let use_min_distance;
            match current_target {
                NpcTarget::Actor(actor) => {
                    if let Some(target_actor) = instance.find_actor(*actor) {
                        target_pos = target_actor.get_common_spawn().position.0;
                    } else {
                        // If we can't find the target actor for some reason (despawn, disconnect, left zone), fall back on a sane-ish destination
                        target_pos = last_position.unwrap_or(spawn.common.position.0);
                    }
                    use_min_distance = true;
                }
                NpcTarget::Position(position) => {
                    target_pos = *position;
                    use_min_distance = false;
                }
            }

            let distance = Vec3A::distance(spawn.common.position.0, target_pos);
            let min_distance = if use_min_distance {
                timeline.attack_range
            } else {
                0.0
            };

            let rotate = |from_pos: Vec3A, to_pos: Vec3A| {
                let rotation = f32::atan2(to_pos.x - from_pos.x, to_pos.z - from_pos.z);
                if rotation >= PI { -PI } else { rotation }
            };

            // If we are in distance, rotate towards target (unless it has to get back near someone first.)
            // Somewhere to be first (near its anchors, behind the tank's target): keep walking there.
            let own_target = match current_target {
                NpcTarget::Actor(actor) => positions.get(actor).copied(),
                NpcTarget::Position(_) => None,
            };
            let strayed = !placement.is_empty()
                && placement
                    .goal(
                        spawn.common.position.0,
                        &positions,
                        &targets,
                        own_target,
                        timeline.attack_range,
                    )
                    .is_some();
            // In reach of who it's after only where it sees them (`in_sight`): out of sight it walks on towards them.
            let in_reach = distance <= min_distance
                && (!use_min_distance
                    || in_sight(
                        instance.collision.as_deref(),
                        spawn.common.position.0,
                        target_pos,
                    ));
            let (position, rotation) = if in_reach && !(strayed && !current_path.is_empty()) {
                // Standing on the spot it walked to keeps its facing (atan2(0, 0) turned it north there.)
                let rotation = if flat_distance(spawn.common.position.0, target_pos) < MIN_STEP {
                    spawn.common.rotation
                } else {
                    rotate(spawn.common.position.0, target_pos)
                };
                (Some(spawn.common.position), Some(rotation))
            } else if !current_path.is_empty() {
                // otherwise, Follow current path
                let next_position = current_path[0];

                let current_position = last_position.unwrap_or(spawn.common.position.0);
                // A step that goes nowhere keeps its facing (atan2(0, 0) turned NPCs north at the start and end of
                // every walk, 2026-10-19 bundle.)
                let rotation = if flat_distance(current_position, next_position) < MIN_STEP {
                    spawn.common.rotation
                } else {
                    rotate(current_position, next_position)
                };

                let along = Vec3A::lerp(current_position, next_position, *current_path_lerp);
                // Walked as a player walks, on the zone's collision: on the floor (not the straight line between
                // two corners of its path, through a hill or over a dip), stopped by walls (not up them), and
                // down off a ledge rather than through the air (Bob's Copperbell Mines bundle of 2026-10-26.)
                let position = match &instance.collision {
                    Some(collision) => {
                        let goes_anywhere = instance.party_bots.contains_key(id)
                            || spawn.common.owner_id.is_valid()
                            || in_content;
                        let max_drop = if goes_anywhere {
                            path_max_drop(spawn.common.position.0, along)
                        } else {
                            OPEN_WORLD_MAX_DROP
                        };
                        let walk = collision.walk(spawn.common.position.0, along, max_drop);
                        let walled_for = instance.walled_ticks.get(id).copied().unwrap_or_default();
                        if walk.blocked {
                            blocked.insert(*id);
                        }
                        // A gimmick the director put up (a sealed room's wall, a locked door) stops it for good, as
                        // it stops a player; so does one between it and where it'd go on to, and a drop there it
                        // wouldn't step down (`past_a_wall`.)
                        let past =
                            (walk.blocked && !walk.sealed && walled_for >= NPC_WALL_PATIENCE_TICKS)
                                .then(|| {
                                    past_a_wall(collision, spawn.common.position.0, along, max_drop)
                                })
                                .flatten();
                        if walk.blocked && walled_for == NPC_WALL_PATIENCE_TICKS {
                            tracing::info!(
                                "NPC {} (layout {}) held up by a wall at {:?} for {} ticks: {}",
                                id.0,
                                spawn.common.layout_id,
                                spawn.common.position.0,
                                walled_for,
                                if past.is_some() {
                                    "walking on through"
                                } else {
                                    "it stays (a gimmick the director put up, or no floor it would step onto, there)"
                                }
                            );
                        }
                        if let Some(past) = past {
                            Position(past)
                        } else {
                            if walk.blocked {
                                walled.insert(*id);
                            }
                            Position(walk.at)
                        }
                    }
                    None => Position(along),
                };
                // Not moved yet (a new path's first tick): nothing to send. Retail's first ActorMove of a walk is
                // already a step; one that only turns it where it stands makes the client play a step on the
                // spot, the "looping walk" of Bob's 2026-10-20 bundle.
                if position == spawn.common.position {
                    (None, None)
                } else {
                    (Some(position), Some(rotation))
                }
            } else {
                (None, None)
            };

            if let NpcTarget::Actor(actor) = current_target {
                target_actor_pos.insert(*actor, target_pos);
            }

            // Only when it moved or turned: an ActorMove to where it already stands makes the client play a step
            // (To Guard a Guardian's masked mage flickered between walking and standing, 2026-10-17 bundle.)
            if let Some(position) = position
                && let Some(rotation) = rotation
                && (position != spawn.common.position
                    || (rotation - spawn.common.rotation).abs() > 0.01)
            {
                // Idle enemies walk about; everyone else runs (retail's speed bytes 20 and 60.)
                let walking = is_walking(state, *returning_home, current_target);
                let (anim_type, speed) = if walking {
                    (MoveAnimationType::WALKING_OR_LANDING, NPC_WALK_SPEED)
                } else {
                    (MoveAnimationType::empty(), NPC_RUN_SPEED)
                };
                actor_moves.push((
                    *id,
                    FromServer::ActorMove(
                        *id,
                        position,
                        rotation,
                        anim_type,
                        MoveAnimationState::None,
                        JumpState::NoneOrFalling,
                        Some((speed * 10.0) as u8),
                    ),
                ));
            }
        }
    }

    // How long each has been stopped by a wall, in a row.
    instance.walled_ticks.retain(|id, _| blocked.contains(id));
    for id in &blocked {
        *instance.walled_ticks.entry(*id).or_default() += 1;
    }

    let mut newly_acquired_targets = Vec::new();
    let mut new_action_requests = Vec::new();
    // Lock-on markers to put over targets as casts begin: (target, Lockon row.)
    let mut new_lockons: Vec<(ObjectId, u32)> = Vec::new();
    let mut new_timeline_states = Vec::new();
    let mut dropped_targets = Vec::new();
    // Idle NPCs that couldn't wander (off the navmesh, or no path to the spot): (actor, layout ID, where.)
    let mut cant_wander: Vec<(ObjectId, u32, Vec3A)> = Vec::new();
    // (NPC, whom it couldn't get to) this tick (`UNREACHABLE_FOR`.)
    let mut unreachable: Vec<(ObjectId, ObjectId)> = Vec::new();
    let now = Instant::now();
    instance.unreachable.retain(|_, until| *until > now);

    // What party bots with a rotation script (`bot_rotation`) see of their targets and surroundings, worked out before
    // the mutable pass: their target's statuses, how many enemies are close to it and to them, how far it is, and
    // their own gauge and actions.
    let mut bot_surroundings: HashMap<ObjectId, BotSurroundings> = HashMap::new();
    for (id, bot) in &instance.party_bots {
        let Some(NetworkedActor::Npc { spawn, .. }) = instance.actors.get(id) else {
            continue;
        };
        if !crate::server::bot_rotation::has(spawn.common.class_job) {
            continue;
        }
        let target = spawn.common.target_id.object_id;
        let target_actor = instance.actors.get(&target);
        let target_statuses = target_actor
            .and_then(|actor| actor.status_effects())
            .map(|effects| {
                effects
                    .data()
                    .iter()
                    .map(|effect| {
                        let left = effects
                            .time_left(effect.effect_id)
                            .map(|left| if left <= 0.0 { 999.0 } else { left })
                            .unwrap_or(0.0);
                        (effect.effect_id, left, effect.source_actor_id == *id)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let here = spawn.common.position.0;
        let there = target_actor.map(|actor| actor.position().0).unwrap_or(here);
        let ground = |a: Vec3A, b: Vec3A| Vec3A::new(a.x - b.x, 0.0, a.z - b.z).length();
        let (mut near_target, mut near_self) = (0, 0);
        for (other_id, other) in &instance.actors {
            if let NetworkedActor::Npc {
                spawn: other_spawn, ..
            } = other
                && matches!(other_spawn.common.object_kind, ObjectKind::BattleNpc(_))
                && other_spawn.common.health_points > 0
                && !instance.party_bots.contains_key(other_id)
                && crate::server::action::at_war(&players_enemies, other)
                && other_spawn
                    .character_data_flags
                    .contains(kawari::ipc::zone::CharacterDataFlag::HOSTILE)
            {
                let at = other_spawn.common.position.0;
                if target_actor.is_some() && ground(at, there) <= BOT_AREA_REACH {
                    near_target += 1;
                }
                if ground(at, here) <= BOT_AREA_REACH {
                    near_self += 1;
                }
            }
        }
        bot_surroundings.insert(
            *id,
            BotSurroundings {
                target_statuses,
                near_target,
                near_self,
                distance: ground(here, there),
                gauge: bot.gauge.named(),
                known: bot.known.clone(),
            },
        );
    }

    // Who sees whom, on the collision as it is this tick (`in_sight`.)
    let sight = instance.collision.clone();
    // mut pass
    for (id, actor) in &mut instance.actors {
        // A director's helper does only what it's told: it neither moves, notices, fights nor follows a timeline
        // (`director:hold_bnpc`; retail's boss helpers stood where they were put and only cast.) Nor does a world bot.
        // Before the duty begins, everyone waits (`Instance::awaiting_commence`.)
        if instance.held_npcs.contains(id)
            || instance.world_bots.contains(id)
            || instance.awaiting_commence
        {
            continue;
        }
        // A bot walking itself is left alone, unless it's waiting out an area attack, where it still acts (it doesn't
        // move: the const pass above leaves it alone.)
        if instance
            .party_bots
            .get(id)
            .is_some_and(|bot| bot.walking_itself && !bot.dodge.is_some_and(|dodge| dodge.holding))
        {
            continue;
        }
        if let NetworkedActor::Npc {
            state,
            navmesh_path: current_path,
            navmesh_path_lerp: current_path_lerp,
            navmesh_target: current_target,
            spawn,
            last_position,
            timeline_position,
            timeline_checked,
            timeline_anchor,
            timeline_hp_done,
            timeline,
            enmity,
            autoattack_timing,
            currently_invulnerable,
            last_wander_timestamp,
            home,
            home_rotation,
            returning_home,
            cast,
            parameters,
            rotation_state,
            placement,
            status_effects,
            pending_action,
            ..
        } = actor
            && *state != NpcState::Dead
            && *state != NpcState::AoeTest
            && spawn.common.health_points > 0
            // A called retainer stands where they appeared until they're sent away.
            && spawn.common.object_kind != ObjectKind::Retainer
        {
            // Where the const pass just moved it: everything below goes from there. From where it stood a tick
            // before, a Brass Blade at Scorpion Crossing that had reached its post (off the navmesh) still looked
            // 0.63 yalms off it, planned the last step again from the mesh's edge, and walked edge, post, edge every
            // half second, facing the same way, forever (Bob's bundle of 2026-10-25.)
            for msg in &actor_moves {
                if let (_, FromServer::ActorMove(msg_id, pos, rotation, ..)) = msg
                    && *id == *msg_id
                {
                    spawn.common.position = *pos;
                    spawn.common.rotation = *rotation;
                }
            }
            if walled.contains(id) {
                *last_position = Some(spawn.common.position.0);
                *current_path_lerp = 0.0;
            }

            // NOTE: this is *intentional* as I believe in retail the timing of actions are dependent on when the actor spawned
            // This doesn't have an effect if you re-aggro them or whatever.
            *timeline_position += 1; // NOTE: change if the length of a server tick changes
            // A pull-relative timeline starts when the fight does, and again at the next one.
            if timeline.from_pull && *state != NpcState::Hate {
                *timeline_position = 0;
                *timeline_checked = 0;
                *timeline_anchor = None;
                *timeline_hp_done = 0;
            }

            // Kept busy by its director: its timeline counts on (above), but it does nothing of its own.
            if instance.busy_npcs.contains_key(id) {
                continue;
            }

            // Enemies stand still (and keep their target) while they cast.
            // (The cast stays recorded afterwards, for the action to land where it was aimed.)
            if cast.is_some_and(|cast| cast.until > Instant::now()) {
                continue;
            }

            // Stunned or asleep: it does nothing at all until that wears off (or, asleep, it's hit.)
            if modifiers::is_incapacitated(status_effects) {
                continue;
            }
            // A party bot just raised moves but doesn't attack (`modifiers::holds_actions`.)
            let holds_actions = modifiers::holds_actions(status_effects);

            // Chased too far from home: give up, go back and heal up, paying no attention to anyone on the way. That's
            // also when the one it's after is that far from its home and it hasn't reached them for a while (out of
            // reach, no attack for `GIVE_UP_UNREACHED_TICKS`): an enemy that can't find a path (no navmesh route, so
            // it never moves) would otherwise keep them in combat forever, wherever they went in the zone. Only then:
            // a Qiqirn Eggdigger hit from 10 yalms by a player standing 41 yalms from its home gave up at once,
            // every time it was hit, and stood still (Bob's 2026-10-22 bundle).
            let from_home = Vec3A::distance(spawn.common.position.0, *home);
            let target_from_home = match current_target {
                Some(NpcTarget::Actor(target)) => enemies
                    .iter()
                    .find(|(id, _, _)| id == target)
                    .filter(|(_, position, _)| {
                        *autoattack_timing >= GIVE_UP_UNREACHED_TICKS
                            && Vec3A::distance(position.0, spawn.common.position.0)
                                > timeline.attack_range
                    })
                    .map(|(_, position, _)| Vec3A::distance(position.0, *home)),
                _ => None,
            };
            // Only out in the world: enemies in content (a quest battle, a duty) fight to the end, as on retail, and the
            // ones there chasing across the arena used to give up and run home at full health mid-fight.
            let is_wild_enemy = !in_content
                && !spawn.common.owner_id.is_valid()
                && spawn.common.object_kind == ObjectKind::BattleNpc(BattleNpcSubKind::Enemy);
            if is_wild_enemy
                && *state == NpcState::Hate
                && (from_home > LEASH_DISTANCE
                    || target_from_home.is_some_and(|distance| distance > LEASH_DISTANCE))
            {
                enmity.clear();
                dropped_targets.push(*id);
                *state = NpcState::natural_state_of(spawn);
                *current_target = Some(NpcTarget::Position(*home));
                spawn.common.target_id = ObjectTypeId::default();
                current_path.clear();
                *current_path_lerp = 0.0;
                *returning_home = true;
                // No instant heal: like retail, it regenerates 10 % a tick on the way (`server_logic_tick`.)
                gave_up.push(*id);
            }
            if *returning_home {
                enmity.clear();
                if from_home <= HOME_DISTANCE {
                    *returning_home = false;
                    *last_wander_timestamp = Instant::now();
                } else if current_target.is_none() {
                    *current_target = Some(NpcTarget::Position(*home));
                }
            }

            // switch to the next node if we passed this one
            if *current_path_lerp >= 1.0 {
                *current_path_lerp = 0.0;
                if !current_path.is_empty() {
                    *last_position = current_path.pop_front();
                }
            }

            // Go after whoever we hate the most (that's still around), sticking with our current target on a tie. Once
            // the fight is over no one does (a damage over time or a heal would start it again.)
            if instance.fight_over {
                enmity.clear();
            }
            enmity.retain(|actor, _| {
                enemies.iter().any(|(id, _, _)| id == actor)
                    && !instance.unreachable.contains_key(&(*id, *actor))
            });
            let current_actor = match current_target {
                Some(NpcTarget::Actor(actor)) if *state == NpcState::Hate => Some(*actor),
                _ => None,
            };
            let most_hated = enmity
                .iter()
                .max_by_key(|(actor, amount)| (**amount, Some(**actor) == current_actor))
                .map(|(actor, _)| *actor);
            if let Some(actor) = most_hated
                && Some(actor) != current_actor
            {
                *state = NpcState::Hate;
                *current_target = Some(NpcTarget::Actor(actor));

                spawn.common.target_id.object_id = actor;
                newly_acquired_targets.push(*id);

                // Head for the new target from where we are, instead of finishing the old path.
                current_path.clear();
                *current_path_lerp = 0.0;
                *autoattack_timing = 0;
            }

            // Striking dummies only take hits: they keep whoever attacks them on their enmity list (so you stay in
            // combat), but never move, turn or fight back, and forget about you once their enmity is reset.
            if spawn.common.name_id == STRIKING_DUMMY_NAME_ID {
                current_path.clear();
                if most_hated.is_none() && current_target.is_some() {
                    if *state == NpcState::Hate {
                        dropped_targets.push(*id);
                    }
                    *current_target = None;
                    *state = NpcState::natural_state_of(spawn);
                    spawn.common.target_id = ObjectTypeId::default();
                }
                continue;
            }

            // Aggressive enemies notice players who come close, whether they're standing around or wandering, unless
            // they're far above them. Guards (character data flags 0x4) notice monsters the same way, never players:
            // retail's Lalafell Brass Blade by Horizon took on a monster 14.5 yalms from his post and one more after
            // it, then walked back (Bob's Chronofoil capture of 2026-10-21).
            let guard = spawn
                .character_data_flags
                .contains(CharacterDataFlag::GUARD);
            if *state == NpcState::Wander
                && !*returning_home
                && !instance.fight_over
                && !instance.unaware_npcs.contains(&spawn.common.layout_id)
                && (guard
                    || spawn
                        .character_data_flags
                        .contains(CharacterDataFlag::HOSTILE))
            {
                let mut game_data = game_data.lock();
                let possible_enemies =
                    game_data.get_battalion_enemies(spawn.common.battalion as u32);

                for (target_id, position, battalion) in &enemies {
                    if !possible_enemies[*battalion as usize] || untargetable.contains(target_id) {
                        continue;
                    }
                    if guard && levels.contains_key(target_id) {
                        continue;
                    }
                    if instance.unreachable.contains_key(&(*id, *target_id)) {
                        continue;
                    }
                    if levels.get(target_id).is_some_and(|level| {
                        *level >= spawn.common.level.saturating_add(AGGRO_LEVEL_GAP)
                    }) {
                        continue;
                    }

                    // Only whom it can see (`in_sight`): a wall or a door the director keeps shut hides them. The Stone
                    // Vigil's 1466 behind the second boss's way on (2001879, shut until the boss falls) noticed the party
                    // through it and pulled them away from the boss to a door they couldn't pass (Bob's bundles
                    // 20261107-214952 and 20261107-221820); on retail it waited until the door opened.
                    if can_notice(spawn.common.position.0, spawn.common.rotation, position.0)
                        && in_sight(sight.as_deref(), spawn.common.position.0, position.0)
                    {
                        enmity.entry(*target_id).or_insert(1);
                        *autoattack_timing = 0;
                        *state = NpcState::Hate;
                        *current_target = Some(NpcTarget::Actor(*target_id));
                        current_path.clear();
                        *current_path_lerp = 0.0;

                        spawn.common.target_id.object_id = *target_id;
                        newly_acquired_targets.push(*id);
                        break;
                    }
                }
            }

            if current_target.is_none() {
                if *state == NpcState::Follow {
                    *current_target = Some(NpcTarget::Actor(spawn.common.owner_id));
                }

                // Duties' NPCs stand where they're put until something starts them (retail's never wander.)
                if *state == NpcState::Wander
                    && !in_content
                    && current_target.is_none()
                    && !instance
                        .zone
                        .stationary_npcs
                        .contains(&spawn.common.layout_id)
                    && Instant::now().duration_since(*last_wander_timestamp) > MOB_WANDER_TIME
                {
                    // Wander around home, rather than drifting further away each time.
                    match instance.navmesh.find_wander_position(*home) {
                        Some(spot) => *current_target = Some(NpcTarget::Position(spot)),
                        None => cant_wander.push((*id, spawn.common.layout_id, *home)),
                    }
                    *last_wander_timestamp = Instant::now()
                        + Duration::from_millis(fastrand::u64(
                            0..=MOB_WANDER_JITTER.as_millis() as u64,
                        ));
                }

                if let NpcState::OnPath { layout_id, index } = *state {
                    if let Some(path) = instance.zone.cached_paths.get(&layout_id) {
                        if let Some(control_point) = path.parent_data.control_points.get(index) {
                            // TODO: take into account the position/rotation/scale of the object
                            *current_target = Some(NpcTarget::Position(Vec3A::from_array(
                                control_point.position,
                            )));
                        } else {
                            tracing::warn!("Control point {index} out of range for {layout_id}!");
                        }
                    } else {
                        tracing::warn!("Failed to find path {layout_id}?");
                    }
                }
            } else if !current_path.is_empty() {
                let next_position = current_path[0];
                let current_position = last_position.unwrap_or(spawn.common.position.0);
                let distance = Vec3A::distance(current_position, next_position);
                let speed = if is_walking(state, *returning_home, current_target.as_ref().unwrap())
                {
                    NPC_WALK_SPEED
                } else {
                    NPC_RUN_SPEED
                };

                *current_path_lerp = f32::clamp(
                    *current_path_lerp + (speed * BEHAVIOR_TICK_SECONDS / distance),
                    0.0,
                    1.0,
                );
            }

            let mut reset_target = false;
            let mut reset_target_for_path = false;
            // Guards, and NPCs that never wander, stand at a post: after a fight they walk back to it.
            let stands_at_post = guard
                || instance
                    .zone
                    .stationary_npcs
                    .contains(&spawn.common.layout_id);
            let was_after_someone = matches!(current_target, Some(NpcTarget::Actor(_)));
            let can_take_action; // FIXME: this is kind of stupid because enemies can do ranged attacks, etc.
            if let Some(current_target) = current_target {
                let target_pos;
                match &current_target {
                    NpcTarget::Actor(actor) => {
                        // Check if the enemy is still valid
                        reset_target = !enemies.iter().any(|(id, _, _)| *id == *actor);
                        if target_actor_pos.contains_key(actor) {
                            target_pos = Some(target_actor_pos[actor]);
                        } else {
                            target_pos = None;
                        }
                    }
                    NpcTarget::Position(position) => {
                        target_pos = Some(*position);
                    }
                }

                // Walking into reach to use a timeline action (`PendingAction`): towards whom it's for, as near as
                // its reach, rather than whom it fights.
                let (target_pos, reach) = match (pending_action.as_ref(), target_pos) {
                    (Some(pending), Some(fighting)) => (
                        Some(positions.get(&pending.on).copied().unwrap_or(fighting)),
                        pending.reach,
                    ),
                    (_, target_pos) => {
                        let size = match current_target {
                            NpcTarget::Actor(actor) => big_hitboxes.get(actor).copied(),
                            NpcTarget::Position(_) => None,
                        };
                        (target_pos, reach_for_size(timeline.attack_range, size))
                    }
                };
                if !reset_target && let Some(target_pos) = target_pos {
                    let distance = Vec3A::distance(spawn.common.position.0, target_pos);
                    // Somewhere it should be first (near its anchors, behind the tank's target): go there.
                    let strayed_to = if placement.is_empty() {
                        None
                    } else {
                        placement.goal(
                            spawn.common.position.0,
                            &positions,
                            &targets,
                            Some(target_pos),
                            timeline.attack_range,
                        )
                    };
                    // In reach: in range, and in sight (`in_sight`): out of sight it walks on towards them (a player
                    // round a corner draws a caster after them, as on retail) and doesn't act on them.
                    let in_reach = distance <= reach
                        && (matches!(current_target, NpcTarget::Position(_))
                            || in_sight(sight.as_deref(), spawn.common.position.0, target_pos));
                    let needs_repath = match current_target {
                        NpcTarget::Actor(_) => {
                            current_path.is_empty() && (!in_reach || strayed_to.is_some())
                        }
                        NpcTarget::Position(_) => current_path.is_empty(),
                    };
                    can_take_action = in_reach && pending_action.is_none();

                    let current_pos = spawn.common.position.0;
                    // Standing in range of who it's after: forget the path, so it sets off from where it stands when
                    // they move. The old one kept advancing while it stood, and it jumped back and forth along it
                    // when it ran again (2026-10-19 bundle.)
                    if matches!(current_target, NpcTarget::Actor(_))
                        && in_reach
                        && strayed_to.is_none()
                        && !current_path.is_empty()
                    {
                        current_path.clear();
                        *last_position = Some(current_pos);
                        *current_path_lerp = 0.0;
                    }
                    // A pet following its owner goes wherever they went: where the navmesh doesn't reach (a hole, or
                    // off its edge), it walks the rest of the way straight. Left to the navmesh, Carbuncle stopped at
                    // the mesh's edge in Lower La Noscea and stepped back and forth there while Kuina stood 17-68
                    // yalms off (Bob's 2026-10-22 bundle.)
                    let following_owner = spawn.common.owner_id.is_valid()
                        && matches!(current_target, NpcTarget::Actor(actor) if *actor == spawn.common.owner_id);
                    if needs_repath {
                        // A chase goes no further than its placement lets it stay (`Placement::chase_goal`.)
                        let chase_to = if matches!(current_target, NpcTarget::Actor(_))
                            && !placement.is_empty()
                        {
                            placement.chase_goal(target_pos, &positions)
                        } else {
                            target_pos
                        };
                        let goal = strayed_to.unwrap_or(chase_to);
                        let gap = if following_owner {
                            f32::INFINITY
                        } else {
                            max_off_mesh_gap
                        };
                        let mut path: VecDeque<Vec3A> = instance
                            .navmesh
                            .calculate_path(current_pos, goal, gap)
                            .into();
                        // In content no path at all (e.g. starting off the navmesh, among a tree's roots) means walking
                        // straight there: dropping the target there sent To Guard a Guardian's second wave wandering
                        // back to where they appeared, around the tree (2026-10-17 bundle).
                        // Party bots likewise, anywhere: the open world's navmesh has holes too (none around
                        // Summerford Farms' houses), and a bot that can't find a path stood still while its party walked
                        // off (Bob's 2026-10-21 bundle: 550 failed paths, both ends off the mesh.)
                        let party_bot = instance.party_bots.contains_key(id);
                        if path.is_empty() && (in_content || party_bot || following_owner) {
                            path.push_back(goal);
                        }
                        let found_path = !path.is_empty();
                        // Detour's path starts where it stands: no step there.
                        while path
                            .front()
                            .is_some_and(|point| flat_distance(*point, current_pos) < MIN_STEP)
                        {
                            path.pop_front();
                        }
                        // Got to the spot it was walking to: it's done. Kept, the spot made it plan a path to where
                        // it stood forever, so it never wandered again and walked on the spot after a chase
                        // (2026-10-19 bundle.) The spot can be off the navmesh (a spawn point by a wall, where the
                        // mesh is eroded), and the path there then starts at the nearest navmesh point and comes
                        // back, so being there is what counts; that walked a Brass Blade by Horizon out and back
                        // every half second, forever (2026-10-20 bundle.)
                        // And a path whose end is no nearer the spot than where it stands gets it no closer: that's as
                        // near as the navmesh goes. From each of two points by a level 6 pugil's spot (its spawn
                        // point, off the mesh) the path's end was the other one, 0.75 yalms off, so it stepped between
                        // them every half second for 16 minutes (Bob's 2026-10-21 bundle.)
                        let no_closer = path.back().is_some_and(|end| {
                            flat_distance(*end, goal) + NO_CLOSER_MARGIN
                                >= flat_distance(current_pos, goal)
                        });
                        let arrived = matches!(current_target, NpcTarget::Position(_))
                            && !matches!(state, NpcState::OnPath { .. })
                            && (flat_distance(current_pos, goal) <= ARRIVE_DISTANCE
                                || (found_path && (path.is_empty() || no_closer)));
                        if arrived {
                            reset_target_for_path = true;
                            path.clear();
                        }

                        *current_path = path.clone();
                        // Start the new path from where we actually are.
                        *last_position = Some(current_pos);
                        *current_path_lerp = 0.0;

                        // Someone it can't get any nearer to, out in the world: it leaves them alone for a while
                        // (`UNREACHABLE_FOR`) instead of noticing them again at once.
                        let open_world_chase = matches!(current_target, NpcTarget::Actor(_))
                            && !in_content
                            && !party_bot
                            && !following_owner;
                        if open_world_chase
                            && (!found_path || (no_closer && distance > timeline.attack_range))
                            && let NpcTarget::Actor(actor) = current_target
                        {
                            unreachable.push((*id, *actor));
                            reset_target = true;
                        }

                        // Drop the current target if we can't path to them too
                        if !found_path && !arrived {
                            reset_target = true;
                            if *state == NpcState::Wander
                                && !*returning_home
                                && matches!(current_target, NpcTarget::Position(_))
                            {
                                cant_wander.push((*id, spawn.common.layout_id, current_pos));
                            }
                        }

                        if let NpcState::OnPath { layout_id, index } = state {
                            *index += 1;
                            // NOTE: Currently loops for now, although we probably don't want that forever
                            if let Some(path) = instance.zone.cached_paths.get(layout_id)
                                && *index + 1 > path.parent_data.control_points.len()
                            {
                                *index = 0;
                            }
                            reset_target_for_path = true;
                        }
                    }
                } else {
                    can_take_action = false;
                }
            } else {
                can_take_action = false;
            }
            // Not for a second after seizing up with paralysis (`server/paralysis.rs`.)
            let paralysed = paralysis_locked.contains(id);
            let at_mounted = mounted_players.contains(&spawn.common.target_id.object_id);
            let can_take_action = can_take_action && !paralysed && !at_mounted && !holds_actions;

            // Only update the timeline on exact second marks. A timeline that always plays (Sastasha's giant clams opening
            // and closing) runs whether or not the NPC is fighting; its actions still need a target. Nothing read the
            // flag before, so the clams never moved (Bob's bundle of 2026-10-23.)
            // A pull-relative timeline (`Timeline::from_pull`): every point due since it last looked, to the half
            // second, on whom it says. One on someone else needs them within the action's range and in sight
            // (`out_of_action_range`.)
            let mut used_timeline = false;
            if timeline.from_pull
                && !paralysed
                && !at_mounted
                && *state == NpcState::Hate
                && spawn.common.target_id.object_id.is_valid()
            {
                // An action it's been walking into reach for (`PendingAction`): used once there, given up after a
                // while; until then nothing else from its timeline comes (it comes after, as after a cast.)
                let ready =
                    pending_action_ready(pending_action, spawn.common.position.0, &positions);
                if ready.is_some() {
                    // It casts where it got to (retail's stayed there, the fighter coming to it.)
                    current_path.clear();
                }
                let mut due: Vec<(TimepointData, bool)> = Vec::new();
                if pending_action.is_none() {
                    let from = *timeline_checked as f32 * BEHAVIOR_TICK_SECONDS;
                    let to = *timeline_position as f32 * BEHAVIOR_TICK_SECONDS;
                    *timeline_checked = *timeline_position;
                    // A health threshold crossed comes first, and only one a tick (it's a cast); one that restarts
                    // the clock takes the timed points' place this tick.
                    let hp = health_percent.get(id).copied().unwrap_or(100.0);
                    let crossed = timeline
                        .timepoints
                        .iter()
                        .enumerate()
                        .find(|(index, point)| {
                            point.hp_below.is_some_and(|below| hp <= below)
                                && *timeline_hp_done & (1 << index) == 0
                        });
                    due = if let Some((index, point)) = crossed {
                        *timeline_hp_done |= 1 << index;
                        vec![(point.data.clone(), point.restarts)]
                    } else {
                        timeline
                            .points_between_after(from, to, *timeline_anchor)
                            .into_iter()
                            .map(|point| (point.data.clone(), point.restarts))
                            .collect()
                    };
                }
                let forced = ready.map(|(data, on)| {
                    due.insert(0, (data, false));
                    on
                });
                for (index, (data, restarts)) in due.into_iter().enumerate() {
                    let forced_on = if index == 0 { forced } else { None };
                    let original = data.clone();
                    if restarts && let TimepointData::Action { action_id, .. } = &data {
                        *timeline_anchor = Some(*action_id);
                        *timeline_position = 0;
                        *timeline_checked = 0;
                    }
                    match data {
                        TimepointData::Action {
                            action_id,
                            lockon,
                            target,
                            facing: faces,
                            reach,
                        } => {
                            let fighting = spawn.common.target_id.object_id;
                            let on = forced_on.unwrap_or_else(|| match target {
                                TimepointTarget::Target => fighting,
                                TimepointTarget::Itself | TimepointTarget::Nobody => *id,
                                TimepointTarget::Other => {
                                    let others: Vec<ObjectId> = enmity
                                        .keys()
                                        .copied()
                                        .filter(|other| *other != fighting)
                                        .collect();
                                    fastrand::choice(&others).copied().unwrap_or(fighting)
                                }
                            });
                            // Too far from whom it's for, or a wall between them: left out
                            // (`out_of_action_range`.)
                            if forced_on.is_none()
                                && reach.is_none()
                                && on != *id
                                && let Some(there) = positions.get(&on).copied()
                            {
                                let range = game_data
                                    .lock()
                                    .get_action_info(action_id)
                                    .map_or(0, |info| info.range);
                                let player = crate::server::action::PLAYER_HITBOX_RADIUS;
                                if out_of_action_range(
                                    range,
                                    flat_distance(spawn.common.position.0, there),
                                    timeline.hitbox_radius.max(player),
                                    big_hitboxes.get(&on).copied().unwrap_or(player),
                                    in_sight(sight.as_deref(), spawn.common.position.0, there),
                                ) {
                                    continue;
                                }
                            }
                            // Out of its reach: it walks there first (`PendingAction`.)
                            if forced_on.is_none()
                                && let Some(reach) = reach
                                && on != *id
                                && positions.get(&on).is_some_and(|at| {
                                    Vec3A::distance(spawn.common.position.0, *at) > reach
                                })
                            {
                                *pending_action = Some(PendingAction {
                                    data: original,
                                    on,
                                    reach,
                                    since: Instant::now(),
                                });
                                current_path.clear();
                                continue;
                            }
                            if let Some(lockon) = lockon {
                                new_lockons.push((on, lockon));
                            }
                            let cast_time_seconds = {
                                let mut game_data = game_data.lock();
                                game_data
                                    .get_casttime(action_id)
                                    .map(|(category, cast_time)| {
                                        parameters.get_adjusted_cast_time(category, cast_time)
                                    })
                                    .unwrap_or(0) as f32
                                    / 1000.0
                            };
                            // It faces whom it casts on as it begins (and keeps that facing, its area's, until it
                            // lands.)
                            let at = positions
                                .get(&on)
                                .copied()
                                .unwrap_or(spawn.common.position.0);
                            // An attack around or in front of itself goes the way it faces: its target's (retail's
                            // Kottos began each Colossal Slam, 28546, facing the tank, -1.94 with the tank at -1.94 and
                            // 1.04 at 0.97, Bob's capture of 2026-10-25. Kawari left it facing wherever its last leap
                            // had, so the cone always went the same way and caught nobody, Bob's bundle of 2026-10-26.)
                            let facing = match (target, faces) {
                                (TimepointTarget::Itself, TimepointFacing::Player) => {
                                    let here = spawn.common.position.0;
                                    player_positions
                                        .iter()
                                        .copied()
                                        .min_by(|a, b| {
                                            flat_distance(here, *a)
                                                .total_cmp(&flat_distance(here, *b))
                                        })
                                        .or_else(|| positions.get(&fighting).copied())
                                }
                                (TimepointTarget::Itself, _) => positions.get(&fighting).copied(),
                                _ if on != *id => Some(at),
                                _ => None,
                            };
                            if let Some(facing) = facing {
                                let here = spawn.common.position.0;
                                if flat_distance(here, facing) >= MIN_STEP {
                                    spawn.common.rotation =
                                        f32::atan2(facing.x - here.x, facing.z - here.z);
                                }
                            }
                            let request = ActionRequest {
                                action_id,
                                action_type: ActionType::Action,
                                rotation1: spawn.common.rotation,
                                target: ObjectTypeId {
                                    object_id: on,
                                    object_type: ObjectTypeKind::None,
                                },
                                ..Default::default()
                            };
                            *cast = Some(NpcCast {
                                until: Instant::now() + Duration::from_secs_f32(cast_time_seconds),
                                length: Duration::from_secs_f32(cast_time_seconds),
                                target_position: at,
                                rotation: spawn.common.rotation,
                                action_id,
                            });
                            new_action_requests.push((*id, request, cast_time_seconds));
                            used_timeline = true;
                        }
                        TimepointData::TimelineState { states } => {
                            new_timeline_states.push((spawn.gimmick_id, states));
                        }
                        TimepointData::Invulnerability { invulnerable } => {
                            *currently_invulnerable = invulnerable;
                        }
                    }
                }
            }

            if !timeline.from_pull
                && (can_take_action || timeline.timeline_always_plays)
                && (*timeline_position % 2) == 0
            {
                // TODO: something worth thinking about is whether to simplify timeline_always_play, and have it always play anyway but skip Action points?

                // NOTE: the "+ 0.5" is a hack to ensure the last timepoint is always counted
                let timeline_position_seconds = *timeline_position / 2;
                let real_timeline_position =
                    timeline_position_seconds as f32 % (timeline.duration() as f32 + 0.5);
                for timepoint in timeline.points_at(real_timeline_position as i32) {
                    match &timepoint.data {
                        TimepointData::Action {
                            action_id, lockon, ..
                        } => {
                            if spawn.common.target_id.object_id.is_valid() && can_take_action {
                                if let Some(lockon) = lockon {
                                    new_lockons.push((spawn.common.target_id.object_id, *lockon));
                                }
                                let (category, cast_time);
                                {
                                    let mut game_data = game_data.lock();
                                    (category, cast_time) =
                                        game_data.get_casttime(*action_id).unwrap();
                                }
                                let adjusted_cast_time =
                                    parameters.get_adjusted_cast_time(category, cast_time);
                                let cast_time_seconds = (adjusted_cast_time as f32) / 1000.0;
                                let request = ActionRequest {
                                    action_id: *action_id,
                                    action_type: ActionType::Action,
                                    rotation1: spawn.common.rotation,
                                    target: spawn.common.target_id,
                                    ..Default::default()
                                };
                                *cast = Some(NpcCast {
                                    until: Instant::now()
                                        + Duration::from_secs_f32(cast_time_seconds),
                                    length: Duration::from_secs_f32(cast_time_seconds),
                                    target_position: target_actor_pos
                                        .get(&spawn.common.target_id.object_id)
                                        .copied()
                                        .unwrap_or(spawn.common.position.0),
                                    rotation: spawn.common.rotation,
                                    action_id: *action_id,
                                });
                                new_action_requests.push((*id, request, cast_time_seconds));
                            }
                        }
                        TimepointData::TimelineState { states } => {
                            // Find the event object bound to our gimmick.
                            let gimmick_id = spawn.gimmick_id;
                            new_timeline_states.push((gimmick_id, states.clone()));
                        }
                        TimepointData::Invulnerability { invulnerable } => {
                            *currently_invulnerable = *invulnerable;
                        }
                    }
                }
            }

            // Its class rotation, checked every tick (it keeps its own pace.)
            let mut used_rotation = used_timeline;
            if can_take_action
                && !used_timeline
                && let Some(rotation) = &timeline.rotation
                && spawn.common.target_id.object_id.is_valid()
            {
                let target = spawn.common.target_id.object_id;
                let hostile = players_enemies
                    .get(spawn.common.battalion as usize)
                    .copied()
                    .unwrap_or(false);
                let allies = if rotation.actions.iter().any(|entry| entry.on_ally.is_some()) {
                    allies_by_base
                        .iter()
                        .filter(|((side, _), (_, ally))| *side == hostile && ally != id)
                        .map(|((_, base), (_, ally))| (*base, *ally))
                        .collect()
                } else {
                    HashMap::new()
                };
                let situation = Situation {
                    now: Instant::now(),
                    target,
                    target_hp: health_percent.get(&target).copied().unwrap_or(100.0),
                    self_hp: health_percent.get(id).copied().unwrap_or(100.0),
                    allies,
                };
                let mut game_data = game_data.lock();
                let mut mechanics = |action| game_data.get_action_mechanics(action);
                // A party bot whose job has a rotation script (`bot_rotation`) uses what that picks; anyone else
                // follows their timeline's rotation.
                let chosen = match bot_surroundings.get(id) {
                    Some(around) => bot_choice(
                        *id,
                        spawn,
                        status_effects,
                        rotation_state,
                        around,
                        &situation,
                        target,
                    ),
                    None => rotation_state
                        .choose(rotation, &situation, &mut mechanics)
                        .map(|action_id| {
                            // Its target, or the ally an `on_ally` entry goes on.
                            let target = rotation
                                .actions
                                .iter()
                                .find(|entry| entry.action == action_id)
                                .and_then(|entry| situation.target_for(entry))
                                .unwrap_or(target);
                            (action_id, target)
                        }),
                };
                if let Some((action_id, target)) = chosen {
                    let target_id = ObjectTypeId {
                        object_id: target,
                        ..spawn.common.target_id
                    };
                    let action_mechanics = mechanics(action_id);
                    rotation_state.record(
                        rotation,
                        action_id,
                        target,
                        situation.now,
                        action_mechanics,
                    );
                    if let Some((category, cast_time)) = game_data.get_casttime(action_id) {
                        let cast_time_seconds =
                            parameters.get_adjusted_cast_time(category, cast_time) as f32 / 1000.0;
                        *cast = Some(NpcCast {
                            until: Instant::now() + Duration::from_secs_f32(cast_time_seconds),
                            length: Duration::from_secs_f32(cast_time_seconds),
                            target_position: target_actor_pos
                                .get(&target)
                                .or_else(|| positions.get(&target))
                                .copied()
                                .unwrap_or(spawn.common.position.0),
                            rotation: spawn.common.rotation,
                            action_id,
                        });
                        let request = ActionRequest {
                            action_id,
                            action_type: ActionType::Action,
                            rotation1: spawn.common.rotation,
                            target: target_id,
                            ..Default::default()
                        };
                        new_action_requests.push((*id, request, cast_time_seconds));
                        used_rotation = true;
                    }
                }
            }

            // Auto-attack every few seconds, counting from when we picked up our target.
            if *state == NpcState::Hate {
                *autoattack_timing += 1;
            }
            // Not as it begins a cast: the auto-attack's own (instant) cast record took the cast's place, so the NPC
            // swung, walked and turned through its cast bar and its area went off facing wherever it turned (Kottos,
            // Bob's bundle of 2026-10-26.)
            if can_take_action
                && !used_rotation
                && cast.is_none_or(|cast| cast.until <= Instant::now())
                && spawn.common.target_id.object_id.is_valid()
                && timeline.autoattack_action_id != 0
                && *autoattack_timing >= NPC_AUTO_ATTACK_TICKS
            {
                *autoattack_timing = 0;
                // Casters' "auto-attacks" are spells with a cast bar.
                let cast_time;
                {
                    let mut game_data = game_data.lock();
                    cast_time = game_data
                        .get_casttime(timeline.autoattack_action_id)
                        .map(|(category, cast_time)| {
                            parameters.get_adjusted_cast_time(category, cast_time)
                        })
                        .unwrap_or(0);
                }
                let request = ActionRequest {
                    action_id: timeline.autoattack_action_id,
                    action_type: ActionType::Action,
                    rotation1: spawn.common.rotation,
                    target: spawn.common.target_id,
                    ..Default::default()
                };
                let cast_time_seconds = cast_time as f32 / 1000.0;
                *cast = Some(NpcCast {
                    until: Instant::now() + Duration::from_secs_f32(cast_time_seconds),
                    length: Duration::from_secs_f32(cast_time_seconds),
                    target_position: target_actor_pos
                        .get(&spawn.common.target_id.object_id)
                        .copied()
                        .unwrap_or(spawn.common.position.0),
                    rotation: spawn.common.rotation,
                    action_id: timeline.autoattack_action_id,
                });
                new_action_requests.push((*id, request, cast_time_seconds));
            }

            if reset_target {
                if *state == NpcState::Hate {
                    dropped_targets.push(*id);
                }
                *current_target = None;
                *state = NpcState::natural_state_of(spawn);
                spawn.common.target_id = ObjectTypeId::default();

                // Back to its post at a walk, still watching (retail's Horizon guard took on a second monster on the
                // way), rather than giving up like a leashed enemy.
                if stands_at_post
                    && was_after_someone
                    && !*returning_home
                    && !in_content
                    && flat_distance(spawn.common.position.0, *home) > ARRIVE_DISTANCE
                {
                    *current_target = Some(NpcTarget::Position(*home));
                    current_path.clear();
                    *current_path_lerp = 0.0;
                }
            }

            if reset_target_for_path {
                // Home at its post: it turns to face the way it did (retail's guard turned back to its spawn
                // rotation on the spot.)
                if stands_at_post
                    && matches!(current_target, Some(NpcTarget::Position(goal)) if flat_distance(*goal, *home) < MIN_STEP)
                    && (spawn.common.rotation - *home_rotation).abs() > 0.01
                {
                    spawn.common.rotation = *home_rotation;
                    actor_moves.push((
                        *id,
                        FromServer::ActorMove(
                            *id,
                            spawn.common.position,
                            *home_rotation,
                            MoveAnimationType::WALKING_OR_LANDING,
                            MoveAnimationState::None,
                            JumpState::NoneOrFalling,
                            Some((NPC_WALK_SPEED * 10.0) as u8),
                        ),
                    ));
                }
                *current_target = None;
            }

            // update common spawn
            for msg in &actor_moves {
                if let (_, FromServer::ActorMove(msg_id, pos, rotation, ..)) = msg
                    && *id == *msg_id
                {
                    spawn.common.position = *pos;
                    spawn.common.rotation = *rotation;
                }
            }
        }
    }

    // Party bots keep to the ground: walking straight where there's no navmesh path, a line between two points went
    // through a slope's rise, 2 yalms under the floor (Bob's Sastasha bundle of 2026-10-23); each step is set on the
    // navmesh's floor under it, where there is one.
    // (With the zone's collision they're on its floor already.)
    for (id, msg) in &mut actor_moves {
        if let FromServer::ActorMove(_, position, ..) = msg
            && instance.collision.is_none()
            && instance.party_bots.contains_key(id)
            && let Some(ground) = instance.navmesh.ground_under(position.0)
        {
            position.0.y = ground.y;
            if let Some(actor) = instance.find_actor_mut(*id) {
                actor.get_common_spawn_mut().position.0.y = ground.y;
            }
        }
    }

    // inform clients of the NPCs new positions
    for (id, msg) in actor_moves {
        let mut network = network.lock();
        network.send_in_range_instance(id, instance, msg, DestinationNetwork::ZoneClients);
    }

    // The lock-on markers, from their targets to everyone around (retail: ActorControl 34 from the tank, the moment the
    // boss's cast began.)
    for (target, lockon) in new_lockons {
        network.lock().send_ac_in_range_inclusive_instance(
            instance,
            target,
            ActorControlCategory::ShowLockonVfx { id: lockon, target },
        );
    }

    // A director that answers what a boss does hears of each action as it begins (`onActorCast`.)
    let begun: Vec<(u32, u32, ObjectId, Option<(Position, ObjectId)>)> = new_action_requests
        .iter()
        .filter_map(|(id, request, _)| {
            let layout_id = instance.find_actor(*id)?.get_common_spawn().layout_id;
            // Where its target stands as it begins (the Thousand Maws of Toto-Rak's last boss's eggs land there), and
            // who it is (the Navel's gaols hold them.)
            let target = instance
                .find_actor(request.target.object_id)
                .map(|target| (target.position(), request.target.object_id));
            Some((layout_id, request.action_id, *id, target))
        })
        .collect();
    if let Some(director) = instance.directors.first_mut() {
        for (layout_id, action_id, id, target) in begun {
            director.on_actor_cast(layout_id, action_id, id, target);
        }
    }

    for (id, request, cast_time) in new_action_requests {
        queue_leap(
            instance,
            id,
            request.action_id,
            request.target.object_id,
            cast_time,
        );
        // No ToggleWeapon after each action: retail draws an NPC's weapon once as it takes someone on (94 ToggleWeapon
        // in the Copperbell Mines' first stretch against Kawari's 523, Bob's captures of 2026-10-25). Kawari drew it
        // again 0.5 s after every action, as NPCs had dropped out of their stance after acting (2026-10-13), probably from
        // the lock (0.6) and swing (0) its NPC ActionEffects carried then, now retail's (`action.rs`.)
        if cast_time == 0.0 {
            instance.insert_task(
                ClientId::default(),
                id,
                Duration::from_secs_f32(0.0),
                QueuedTaskData::CastAction {
                    request: request.clone(),
                    interruptible: false,
                },
            );
        } else {
            // The cast's position is where it's aimed, and one placed on the ground names no target (`cast_aim`.)
            let aim = retail_cast_aim(
                request.action_id,
                game_data
                    .lock()
                    .get_action_info(request.action_id)
                    .map(|info| info.cast_type),
            );
            let (cast_target, position) = {
                let actor = instance.find_actor(id).unwrap();
                let aimed_at = match actor {
                    NetworkedActor::Npc {
                        cast: Some(cast), ..
                    } => cast.target_position,
                    _ => actor.position().0,
                };
                if aim == CastAim::Caster {
                    (id, actor.position())
                } else {
                    cast_aim(aim, request.target.object_id, aimed_at)
                }
            };

            // inform players that this enemy is casting
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
                spell_id: request.action_id as u16,
                action_type: request.action_type,
                omen_delay: 0,
                action_id: request.action_id,
                cast_time: npc_cast_bar_time(cast_time),
                target: cast_target,
                rotation: request.rotation1,
                interruptible: false,
                ballista_entity_id: ObjectId::default(),
                position,
            });

            let mut network = network.lock();
            network.send_in_range_instance(
                id,
                instance,
                FromServer::PacketSegment(ipc, id),
                DestinationNetwork::ZoneClients,
            );

            instance.insert_task(
                ClientId::default(),
                id,
                Duration::from_secs_f32(cast_time),
                QueuedTaskData::CastAction {
                    request: request.clone(),
                    interruptible: false, // TODO
                },
            );
        }
    }

    for (gimmick_id, states) in new_timeline_states {
        let actor_id;
        {
            actor_id = instance.find_object_by_bind_layout_id(gimmick_id);
        }
        if let Some(actor_id) = actor_id {
            let mut network = network.lock();
            set_shared_group_timeline_state(instance, &mut network, actor_id, &states);
        }
    }

    // Say which NPCs stand still because the navmesh doesn't let them wander, once each, and only the first few.
    for (id, layout_id, position) in cant_wander {
        if instance.stuck_wanderers.insert(id) {
            let count = instance.stuck_wanderers.len();
            if count <= STUCK_WANDERERS_LOGGED {
                tracing::warn!(
                    "NPC {layout_id} can't wander from {position:?} in zone {}: no navmesh path there",
                    instance.zone.id
                );
            } else if count == STUCK_WANDERERS_LOGGED + 1 {
                tracing::warn!(
                    "More NPCs can't wander in zone {} (not logging the rest)",
                    instance.zone.id
                );
            }
        }
    }

    // Nobody claims those that gave up a chase any more.
    for id in gave_up {
        first_attack::clear(&mut network.lock(), instance, id);
    }

    for pair in unreachable {
        instance.unreachable.insert(pair, now + UNREACHABLE_FOR);
    }

    // Those that lost their target sheathe their weapon, unless they find another one in the meantime.
    for id in dropped_targets {
        instance.insert_task(
            ClientId::default(),
            id,
            NPC_SHEATHE_DELAY,
            QueuedTaskData::SheatheWeapon,
        );
    }

    // create hate list
    for (id, actor) in &instance.actors {
        if let NetworkedActor::Npc {
            state,
            navmesh_target: current_target,
            spawn,
            enmity,
            ..
        } = actor
        {
            if out_of_the_fight(instance, *id, state) {
                continue;
            }

            // Everyone on its enmity list has it as a hater, whoever it's going for: retail's HaterList kept each
            // enemy that had the player on its list, at 0-1 % while the tank held it (Bob's Copperbell Mines capture
            // with Duty Support, 2026-10-25). Counted from its target only, an enemy left the player's list the
            // moment the tank took it, and was always at 100 % while it was there.
            for hated in enmity.keys() {
                let list = haters.entry(*hated).or_default();
                if !list.contains(id) {
                    list.push(*id);
                }
            }

            if let Some(current_target) = current_target
                && let NpcTarget::Actor(actor) = current_target
            {
                if newly_acquired_targets.contains(id) {
                    // Send an ACT for a visual indicator, and stuff.
                    let mut network = network.lock();
                    let target = ObjectTypeId {
                        object_id: *actor,
                        object_type: ObjectTypeKind::None,
                    };
                    network.send_in_range_instance(
                        *id,
                        instance,
                        FromServer::ActorControlTarget(
                            *id,
                            target,
                            ActorControlCategory::SetTarget,
                        ),
                        DestinationNetwork::ZoneClients,
                    );

                    // TODO: does this need to be set somewhere in CommonSpawn too?
                    network.send_ac_in_range_instance(
                        instance,
                        *id,
                        ActorControlCategory::SetBattle { battle: true },
                    );

                    // Draw their weapon, so they stay in their battle stance between attacks.
                    network.send_ac_in_range_instance(
                        instance,
                        *id,
                        ActorControlCategory::ToggleWeapon {
                            shown: true,
                            unk_flag: 1,
                            unk2: 0,
                        },
                    );

                    if let Some(director) = &mut instance.directors.first_mut() {
                        director.on_actor_aggro(spawn.common.layout_id, *actor);
                    }
                }

                let list = haters.entry(*actor).or_default();
                if !list.contains(id) {
                    list.push(*id);
                }
            }
        }
    }
}

/// Whether an NPC shows no fight yet: neither its target nor its battle stance, nor a place on anyone's hater list (so no
/// one is put in combat by it). The dead and a director's helpers (`hold_bnpc`); and everyone before the duty begins
/// (`Instance::awaiting_commence`): retail's player was out of combat through Notorious Biggs' opening (Bob's
/// Chronofoil capture 689b15da), Kawari's in combat before it started (Bob's bundle 20261111-194208.)
pub(crate) fn out_of_the_fight(instance: &Instance, id: ObjectId, state: &NpcState) -> bool {
    *state == NpcState::Dead
        || *state == NpcState::AoeTest
        || instance.held_npcs.contains(&id)
        || instance.awaiting_commence
}

/// A party wipe in a duty (`server/wipe.rs`): every enemy that was fighting, or hurt, goes back to where it began, whole,
/// forgetting whom it hated, out of battle, its cast and its director's hold on it (`busy_bnpc`, `root_bnpc`) gone, as
/// everyone is sent back to the start. Retail (the Stone Vigil, Bob's Chronofoil capture 1cefedf4 of 2026-11-07, three
/// wipes): the bosses were back where they began at full health when the party returned, and fought anew when pulled;
/// Kawari's first boss stayed in battle with them and chased the party to the entrance (Bob's bundle 20261107-212704).
/// Returns the NPCs reset.
/// Who an NPC never notices: whoever can't be targeted (DisplayFlag 0x8 in their spawn, as retail sends it, or made so
/// by a director's `set_targetable`). The Howling Eye's monoliths (237, display flags 0x40208) are only in the way of
/// Garuda's attacks, and Kawari's Garuda went after one as soon as she was out (Bob's bundle 20261111-125206); retail's
/// chose a player or a Duty Support member every time (Bob's Chronofoil captures 6a5244ec, 09ecf8e7, 6c495fc4). A
/// director's `set_hate` still sets one on them (the razor plumes go for the monoliths, as retail's did.)
pub(crate) fn unnoticed(instance: &Instance) -> HashSet<ObjectId> {
    instance
        .actors
        .iter()
        .filter(|(id, actor)| {
            instance.untargetable_npcs.contains(id)
                || matches!(actor, NetworkedActor::Npc { spawn, .. }
                    if spawn.common.display_flags.contains(kawari::ipc::zone::DisplayFlag::UNK2))
        })
        .map(|(id, _)| *id)
        .collect()
}

/// `players_enemies`: the battalions at war with the players' (`GameData::get_battalion_enemies`).
pub fn reset_after_wipe(
    network: &mut NetworkState,
    players_enemies: &[bool],
    instance: &mut Instance,
) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = instance
        .actors
        .iter()
        // NPCs only, before anything asks for their spawn: an event object's or a coffer's has none (asking brought the
        // world server down at a wipe in the Stone Vigil, Bob's bundle 20261107-215105.)
        .filter(|(id, actor)| {
            matches!(actor, NetworkedActor::Npc { state, enmity, spawn, .. }
                if *state != NpcState::Dead
                    && spawn.common.health_points > 0
                    && (*state == NpcState::Hate
                        || !enmity.is_empty()
                        || spawn.common.health_points < spawn.common.max_health_points))
                && !instance.party_bots.contains_key(id)
                && crate::server::action::at_war(players_enemies, actor)
        })
        .map(|(id, _)| *id)
        .collect();
    for id in &ids {
        instance.busy_npcs.remove(id);
        instance.rooted_npcs.remove(id);
        let Some(NetworkedActor::Npc {
            spawn,
            state,
            enmity,
            navmesh_target,
            navmesh_path,
            navmesh_path_lerp,
            last_position,
            cast,
            home,
            home_rotation,
            returning_home,
            autoattack_timing,
            ..
        }) = instance.actors.get_mut(id)
        else {
            continue;
        };
        enmity.clear();
        *state = NpcState::natural_state_of(spawn);
        *navmesh_target = None;
        navmesh_path.clear();
        *navmesh_path_lerp = 0.0;
        *last_position = Some(*home);
        *cast = None;
        *returning_home = false;
        *autoattack_timing = 0;
        spawn.common.target_id = ObjectTypeId::default();
        spawn.common.position = Position(*home);
        spawn.common.rotation = *home_rotation;
        spawn.common.health_points = spawn.common.max_health_points;
        let (position, rotation) = (spawn.common.position, spawn.common.rotation);
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
            position,
            rotation,
            ..Default::default()
        }));
        network.send_to_instance(
            ObjectId::default(),
            instance,
            FromServer::PacketSegment(ipc, *id),
            DestinationNetwork::ZoneClients,
        );
        network.send_ac_in_range_instance(
            instance,
            *id,
            ActorControlCategory::SetBattle { battle: false },
        );
        network.send_in_range_instance(
            *id,
            instance,
            FromServer::ActorControlTarget(
                *id,
                ObjectTypeId::default(),
                ActorControlCategory::SetTarget,
            ),
            DestinationNetwork::ZoneClients,
        );
    }
    ids
}

/// The fight is over (a quest battle won): every NPC stops fighting and stays out of it. Retail, the moment
/// Victory in Peril's boss fell: the allies sheathed and left battle (ActorControl ToggleWeapon 0, SetBattle 0) and
/// dropped their targets half a second later (ActorControlTarget SetTarget 0), and the player left battle (Bob's
/// Chronofoil capture of 2026-10-21, the won run). Enemies still standing (none were in that run) stop the same way;
/// left to it, the last two archers killed the allied conjurer and shot the player through the closing scene (Bob's
/// bundle of 2026-10-22.)
pub fn end_fight(instance: &mut Instance, network: &mut NetworkState) {
    let mut npcs = Vec::new();
    let mut players = Vec::new();
    for (id, actor) in &mut instance.actors {
        match actor {
            NetworkedActor::Npc {
                spawn,
                state,
                enmity,
                navmesh_target,
                navmesh_path,
                cast,
                ..
            } => {
                if *state == NpcState::Dead {
                    continue;
                }
                enmity.clear();
                *state = NpcState::natural_state_of(spawn);
                *navmesh_target = None;
                navmesh_path.clear();
                *cast = None;
                spawn.common.target_id = ObjectTypeId::default();
                npcs.push(*id);
            }
            NetworkedActor::Player { .. } => players.push(*id),
            _ => {}
        }
    }
    instance.fight_over = true;
    for id in npcs {
        network.send_ac_in_range_instance(
            instance,
            id,
            ActorControlCategory::ToggleWeapon {
                shown: false,
                unk_flag: 1,
                unk2: 1,
            },
        );
        network.send_ac_in_range_instance(
            instance,
            id,
            ActorControlCategory::SetBattle { battle: false },
        );
        network.send_in_range_instance(
            id,
            instance,
            FromServer::ActorControlTarget(
                id,
                ObjectTypeId::default(),
                ActorControlCategory::SetTarget,
            ),
            DestinationNetwork::ZoneClients,
        );
    }
    // Retail sent the player's as an ActorControl about themselves, to them too.
    for id in players {
        network.send_in_range_inclusive_instance(
            id,
            instance,
            FromServer::ActorControl(id, ActorControlCategory::SetBattle { battle: false }),
            DestinationNetwork::ZoneClients,
        );
    }
}

/// Whether an NPC is walking rather than running: idle enemies wandering to a spot near home, and patrols.
fn is_walking(state: &NpcState, returning_home: bool, target: &NpcTarget) -> bool {
    match state {
        // Patrols walk their route.
        NpcState::OnPath { .. } => true,
        NpcState::Wander => !returning_home && matches!(target, NpcTarget::Position(_)),
        _ => false,
    }
}

/// Whether an aggressive enemy at `from`, facing `rotation`, notices someone at `to`: within `AGGRO_PROXIMITY` all
/// round, or within `AGGRO_RANGE` in its sight cone.
fn can_notice(from: Vec3A, rotation: f32, to: Vec3A) -> bool {
    let distance = Vec3A::distance(from, to);
    if distance < AGGRO_PROXIMITY {
        return true;
    }
    if distance >= AGGRO_RANGE {
        return false;
    }
    let towards = f32::atan2(to.x - from.x, to.z - from.z);
    let mut difference = (towards - rotation).abs() % (2.0 * PI);
    if difference > PI {
        difference = 2.0 * PI - difference;
    }
    difference <= AGGRO_SIGHT_HALF_ANGLE
}

#[cfg(test)]
mod notice_tests {
    use super::*;

    #[test]
    fn sight_and_proximity() {
        let enemy = Vec3A::ZERO;
        // Facing +z (rotation 0.)
        assert!(can_notice(enemy, 0.0, Vec3A::new(0.0, 0.0, 14.0)));
        assert!(!can_notice(enemy, 0.0, Vec3A::new(0.0, 0.0, -14.0)));
        assert!(can_notice(enemy, 0.0, Vec3A::new(0.0, 0.0, -10.0)));
        assert!(!can_notice(enemy, 0.0, Vec3A::new(14.0, 0.0, 0.0)));
        assert!(can_notice(enemy, PI / 2.0, Vec3A::new(14.0, 0.0, 0.0)));
        assert!(!can_notice(enemy, 0.0, Vec3A::new(0.0, 0.0, 16.0)));
        // Across the ±π seam.
        assert!(can_notice(enemy, PI - 0.1, Vec3A::new(-1.0, 0.0, -13.0)));
    }
}

#[cfg(test)]
mod wall_tests {
    use super::*;
    use crate::collision::Collision;

    /// Bob's marks of 2026-11-14 (Every Little Thing She Does Is Magitek): the lancer used range-3 weaponskills on
    /// the reaper from 33 yalms, through a wall. Retail's stood 4.5 from it.
    #[test]
    fn a_timeline_action_needs_whom_it_s_for_in_its_range_and_sight() {
        let player = crate::server::action::PLAYER_HITBOX_RADIUS;
        // The lancer's weaponskills (Action.Range -1: its weapon's) at 33 yalms, and at retail's 4.5.
        assert!(out_of_action_range(-1, 33.0, player, 1.5, true));
        assert!(!out_of_action_range(-1, 4.5, player, 1.5, true));
        // In range but through a wall.
        assert!(out_of_action_range(-1, 4.5, player, 1.5, false));
        // A spell of 25 yalms on someone 20 off still comes; beyond its range it doesn't.
        assert!(!out_of_action_range(25, 20.0, player, player, true));
        assert!(out_of_action_range(25, 40.0, player, player, true));
        // One centred on its user has no range to be out of.
        assert!(!out_of_action_range(0, 33.0, player, player, false));
    }

    #[test]
    fn going_on_past_a_wall_keeps_off_a_drop_and_a_shut_door() {
        // A floor at 0 for x < 6, the sunken one at -3.2 beyond, a wall at x = 2 (z -10..10) and a door the director
        // shows shut (gimmick 9) at x = -2.
        let mut v: Vec<[f32; 3]> = Vec::new();
        let mut t: Vec<[u32; 3]> = Vec::new();
        fn quad(v: &mut Vec<[f32; 3]>, t: &mut Vec<[u32; 3]>, corners: [[f32; 3]; 4]) {
            let i = v.len() as u32;
            v.extend(corners);
            t.extend([[i, i + 1, i + 2], [i, i + 2, i + 3]]);
        }
        quad(
            &mut v,
            &mut t,
            [
                [-10.0, 0.0, -10.0],
                [6.0, 0.0, -10.0],
                [6.0, 0.0, 10.0],
                [-10.0, 0.0, 10.0],
            ],
        );
        quad(
            &mut v,
            &mut t,
            [
                [6.0, -3.2, -10.0],
                [20.0, -3.2, -10.0],
                [20.0, -3.2, 10.0],
                [6.0, -3.2, 10.0],
            ],
        );
        quad(
            &mut v,
            &mut t,
            [
                [2.0, 0.0, -10.0],
                [2.0, 0.0, 10.0],
                [2.0, 4.0, 10.0],
                [2.0, 4.0, -10.0],
            ],
        );
        let plain = t.len();
        quad(
            &mut v,
            &mut t,
            [
                [-2.0, 0.0, -10.0],
                [-2.0, 0.0, 10.0],
                [-2.0, 4.0, 10.0],
                [-2.0, 4.0, -10.0],
            ],
        );
        let mut owners = vec![0; plain];
        owners.resize(t.len(), 9);
        let mut collision = Collision::build(v, t, owners, Vec::new(), 0.75, 0.4, 55.0);
        let here = Vec3A::new(0.0, 0.0, 0.0);
        // On past the wall to the same floor: yes.
        assert_eq!(
            past_a_wall(
                &collision,
                here,
                Vec3A::new(4.0, 0.0, 0.0),
                PATH_DROP_MARGIN
            ),
            Some(Vec3A::new(4.0, 0.0, 0.0))
        );
        // Over the sunken floor, deeper than it would step down: no.
        assert_eq!(
            past_a_wall(
                &collision,
                here,
                Vec3A::new(8.0, 0.0, 0.0),
                PATH_DROP_MARGIN
            ),
            None
        );
        // Through the door: until the director shuts it.
        let beyond = Vec3A::new(-4.0, 0.0, 0.0);
        assert!(past_a_wall(&collision, here, beyond, PATH_DROP_MARGIN).is_some());
        collision.set_gimmick_sealed(9, true);
        assert_eq!(
            past_a_wall(&collision, here, beyond, PATH_DROP_MARGIN),
            None
        );
        // And it sees no one through the wall.
        assert!(!in_sight(Some(&collision), here, Vec3A::new(4.0, 0.0, 0.0)));
        assert!(in_sight(None, here, Vec3A::new(4.0, 0.0, 0.0)));
    }
}

/// If `action_id` is a leap (`NPC_LEAPS`), lands `actor_id` beside `target_id` when retail's would.
pub fn queue_leap(
    instance: &mut Instance,
    actor_id: ObjectId,
    action_id: u32,
    target_id: ObjectId,
    cast_time: f32,
) {
    if let Some((_, after)) = NPC_LEAPS.iter().find(|(leap, _)| *leap == action_id)
        && target_id != actor_id
        && target_id.is_valid()
    {
        instance.insert_task(
            ClientId::default(),
            actor_id,
            Duration::from_secs_f32(cast_time + after),
            QueuedTaskData::LeapTo { target: target_id },
        );
    }
}

/// An NPC's leap lands: `SnapNextMove`, then an ActorMove to `LEAP_SHORT_OF` yalms short of `target_id` (on the floor
/// there), facing it, as retail's Duty Support Marauder's 29291 (see `NPC_LEAPS`.) Its path starts again from there.
pub fn leap_to(
    network: &mut NetworkState,
    instance: &mut Instance,
    actor_id: ObjectId,
    target_id: ObjectId,
) {
    let Some(target) = instance
        .find_actor(target_id)
        .map(|actor| actor.position().0)
    else {
        return;
    };
    let collision = instance.collision.clone();
    let landing;
    let rotation;
    {
        let Some(NetworkedActor::Npc {
            spawn,
            navmesh_path,
            navmesh_path_lerp,
            last_position,
            ..
        }) = instance.find_actor_mut(actor_id)
        else {
            return;
        };
        if spawn.common.health_points == 0 {
            return;
        }
        let here = spawn.common.position.0;
        let way = Vec3A::new(target.x - here.x, 0.0, target.z - here.z);
        let distance = way.length();
        if distance <= LEAP_SHORT_OF {
            return;
        }
        rotation = f32::atan2(way.x, way.z);
        let mut spot = here + way * ((distance - LEAP_SHORT_OF) / distance);
        spot.y = target.y;
        landing = match &collision {
            Some(collision) => collision.settle(spot),
            None => spot,
        };
        spawn.common.position = Position(landing);
        spawn.common.rotation = rotation;
        navmesh_path.clear();
        *navmesh_path_lerp = 0.0;
        *last_position = Some(landing);
    }
    if let Some(bot) = instance.party_bots.get_mut(&actor_id) {
        bot.path.clear();
    }
    network.send_ac_in_range_instance(instance, actor_id, ActorControlCategory::SnapNextMove);
    network.send_in_range_instance(
        actor_id,
        instance,
        FromServer::ActorMove(
            actor_id,
            Position(landing),
            rotation,
            MoveAnimationType::empty(),
            MoveAnimationState::None,
            JumpState::NoneOrFalling,
            Some(LEAP_MOVE_SPEED),
        ),
        DestinationNetwork::ZoneClients,
    );
}

/// Has the NPC `actor_id` use `action_id` on `target_id` now, as a director's `use_action` does (and party bots'
/// heals): it stands still while it casts (its area lands where it was aimed now), shows its cast bar, and the action
/// lands when the cast ends.
/// The Action sheet's CastType of an area placed on the ground where it was aimed (`ActionInfo::area_shape`.)
const CAST_TYPE_PLACED: u8 = 7;

/// What an NPC's ActorCast says of its aim, as retail's did (1,362 casts in Bob's two Stone Vigil captures, 1cefedf4
/// and 341808b0): its position is where the cast is aimed, never the caster's own unless it's cast on itself (the
/// target's spot in 1,164 of 1,169 casts on someone, the caster's in all 127 on itself), and an area placed on the
/// ground names no target (0xE0000000, all 66: Isgebind's Sheet of Ice 1023 and its helpers' 1024, 738, 29053) with
/// the spot as its position, which is where the client draws its marker. Kawari sent the caster's position and the
/// target's ID, so the Stone Vigil's helpers' markers were all drawn where the helpers stood, in the middle of the
/// room, while they hit where the party had stood (Bob's bundle 20261110-182408.)
pub fn cast_aim(aim: CastAim, target: ObjectId, aimed_at: Vec3A) -> (ObjectId, Position) {
    (
        if aim == CastAim::Ground {
            ObjectId::default()
        } else {
            target
        },
        Position(aimed_at),
    )
}

/// What an NPC's cast names (`cast_aim`): its target, itself, or no one (an area placed on the ground.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastAim {
    Target,
    Caster,
    Ground,
}

/// How retail's NPCs aimed `action`'s cast (`resources/data/cast_aims.json`: every NPC cast in Bob's 124 Chronofoil
/// captures, 805 actions, all but three always the same way: 609 on the caster itself, 97 on no one, 99 at someone),
/// or else by its CastType: placed on the ground (7) names no one, anything else its target. Before 2026-11-11 only the
/// CastType decided, so an area round the caster named its target and the client drew its marker there: History
/// Repeating's lancers' 866, retail's on themselves 437 times of 437 (Bob's bundle 20261111-072219: "some of the enemies
/// didn't have their AOEs correctly set".)
pub fn retail_cast_aim(action: u32, cast_type: Option<u8>) -> CastAim {
    static AIMS: std::sync::OnceLock<HashMap<u32, CastAim>> = std::sync::OnceLock::new();
    let aims = AIMS.get_or_init(|| {
        let value: serde_json::Value =
            serde_json::from_str(include_str!("../../../../resources/data/cast_aims.json"))
                .unwrap_or_default();
        let mut aims = HashMap::new();
        for (key, aim) in [("self", CastAim::Caster), ("none", CastAim::Ground)] {
            for id in value
                .get(key)
                .and_then(|ids| ids.as_array())
                .into_iter()
                .flatten()
                .filter_map(|id| id.as_u64())
            {
                aims.insert(id as u32, aim);
            }
        }
        aims
    });
    aims.get(&action).copied().unwrap_or(match cast_type {
        Some(CAST_TYPE_PLACED) => CastAim::Ground,
        _ => CastAim::Target,
    })
}

pub fn npc_use_action(
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
    actor_id: ObjectId,
    action_id: u32,
    target_id: ObjectId,
) {
    let Some(actor) = instance.find_actor(actor_id) else {
        return;
    };
    // Just raised: no action until its protection ends (`modifiers::holds_actions`: a party bot's heals, raises and
    // rotation all come here.)
    if actor.status_effects().is_some_and(modifiers::holds_actions) {
        return;
    }
    let position = actor.position();
    let rotation = actor.rotation();
    let cast_time = {
        let mut game_data = game_data.lock();
        game_data
            .get_casttime(action_id)
            .map(|(_, cast_time)| cast_time)
            .unwrap_or(0) as f32
            / 1000.0
    };
    let request = ActionRequest {
        action_id,
        action_type: ActionType::Action,
        rotation1: rotation,
        target: ObjectTypeId {
            object_id: target_id,
            object_type: ObjectTypeKind::None,
        },
        ..Default::default()
    };

    let target_position = instance
        .find_actor(target_id)
        .map(|actor| actor.position().0)
        .unwrap_or(position.0);
    if let Some(NetworkedActor::Npc { cast, .. }) = instance.find_actor_mut(actor_id) {
        *cast = Some(NpcCast {
            until: Instant::now() + Duration::from_secs_f32(cast_time),
            length: Duration::from_secs_f32(cast_time),
            target_position,
            rotation,
            action_id,
        });
    }

    // Show the cast bar, the same way NPCs cast their own actions: aimed where its target stands (`cast_aim`.)
    if cast_time > 0.0 {
        let aim = retail_cast_aim(
            action_id,
            game_data
                .lock()
                .get_action_info(action_id)
                .map(|info| info.cast_type),
        );
        let (cast_target, position) = if aim == CastAim::Caster {
            (actor_id, Position(position.0))
        } else {
            cast_aim(aim, target_id, target_position)
        };
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
            spell_id: request.action_id as u16,
            action_type: request.action_type,
            omen_delay: 0,
            action_id: request.action_id,
            cast_time: npc_cast_bar_time(cast_time),
            target: cast_target,
            rotation,
            interruptible: false,
            ballista_entity_id: ObjectId::default(),
            position,
        });
        let mut network = network.lock();
        network.send_in_range_instance(
            actor_id,
            instance,
            FromServer::PacketSegment(ipc, actor_id),
            DestinationNetwork::ZoneClients,
        );
    }

    queue_leap(instance, actor_id, action_id, target_id, cast_time);
    instance.insert_task(
        ClientId::default(),
        actor_id,
        Duration::from_secs_f32(cast_time),
        QueuedTaskData::CastAction {
            request,
            interruptible: cast_time > 0.0,
        },
    );
}
