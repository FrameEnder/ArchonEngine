use std::{
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    sync::Arc,
    time::{Duration, Instant},
};

use glam::Vec3A;
use kawari::{
    common::{
        AUTO_ATTACK_RATE, JumpState, MOB_WANDER_JITTER, MOB_WANDER_TIME, MoveAnimationState,
        MoveAnimationType, ObjectId, ObjectTypeId, ObjectTypeKind, Position,
        STRIKING_DUMMY_NAME_ID, TimepointData,
    },
    ipc::zone::{
        ActionRequest, ActionType, ActorControlCategory, BattleNpcSubKind, CharacterDataFlag,
        ObjectKind, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer, GameData,
    gamedata::AreaShape,
    navmesh::MAX_OFF_MESH_GAP,
    server::{
        actor::{NetworkedActor, NpcCast, NpcState, NpcTarget, set_shared_group_timeline_state},
        first_attack,
        instance::{Instance, QueuedTaskData},
        modifiers,
        network::{DestinationNetwork, NetworkState},
        rotation::Situation,
    },
};

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
/// How many NPCs that can't wander an instance logs (the rest are only counted.)
const STUCK_WANDERERS_LOGGED: usize = 5;

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

    // const pass
    for (id, actor) in &instance.actors {
        // A director's helper does only what it's told (`director:hold_bnpc`.)
        if instance.held_npcs.contains(id) {
            continue;
        }
        // A party bot out of combat walks itself (server/party_bot.rs.)
        if instance
            .party_bots
            .get(id)
            .is_some_and(|bot| bot.walking_itself)
        {
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
            && !cast.is_some_and(|cast| cast.until > Instant::now())
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
            let (position, rotation) = if distance <= min_distance
                && !(strayed && !current_path.is_empty())
            {
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

                let position = Position(Vec3A::lerp(
                    current_position,
                    next_position,
                    *current_path_lerp,
                ));
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

    // mut pass
    for (id, actor) in &mut instance.actors {
        // A director's helper does only what it's told: it neither moves, notices, fights nor follows a timeline
        // (`director:hold_bnpc`; retail's boss helpers stood where they were put and only cast.)
        if instance.held_npcs.contains(id) {
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

            // NOTE: this is *intentional* as I believe in retail the timing of actions are dependent on when the actor spawned
            // This doesn't have an effect if you re-aggro them or whatever.
            *timeline_position += 1; // NOTE: change if the length of a server tick changes

            // Enemies stand still (and keep their target) while they cast.
            // (The cast stays recorded afterwards, for the action to land where it was aimed.)
            if cast.is_some_and(|cast| cast.until > Instant::now()) {
                continue;
            }

            // Stunned or asleep: it does nothing at all until that wears off (or, asleep, it's hit.)
            if modifiers::is_incapacitated(status_effects) {
                continue;
            }

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
                    if !possible_enemies[*battalion as usize] {
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

                    if can_notice(spawn.common.position.0, spawn.common.rotation, position.0) {
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
                    let needs_repath = match current_target {
                        NpcTarget::Actor(_) => {
                            current_path.is_empty()
                                && (distance > timeline.attack_range || strayed_to.is_some())
                        }
                        NpcTarget::Position(_) => current_path.is_empty(),
                    };
                    can_take_action = distance <= timeline.attack_range;

                    let current_pos = spawn.common.position.0;
                    // Standing in range of who it's after: forget the path, so it sets off from where it stands when
                    // they move. The old one kept advancing while it stood, and it jumped back and forth along it
                    // when it ran again (2026-10-19 bundle.)
                    if matches!(current_target, NpcTarget::Actor(_))
                        && distance <= timeline.attack_range
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
                        let goal = strayed_to.unwrap_or(target_pos);
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

            // Only update the timeline on exact second marks. A timeline that always plays (Sastasha's giant clams opening
            // and closing) runs whether or not the NPC is fighting; its actions still need a target. Nothing read the
            // flag before, so the clams never moved (Bob's bundle of 2026-10-23.)
            if (can_take_action || timeline.timeline_always_plays) && (*timeline_position % 2) == 0
            {
                // TODO: something worth thinking about is whether to simplify timeline_always_play, and have it always play anyway but skip Action points?

                // NOTE: the "+ 0.5" is a hack to ensure the last timepoint is always counted
                let timeline_position_seconds = *timeline_position / 2;
                let real_timeline_position =
                    timeline_position_seconds as f32 % (timeline.duration() as f32 + 0.5);
                for timepoint in timeline.points_at(real_timeline_position as i32) {
                    match &timepoint.data {
                        TimepointData::Action { action_id, lockon } => {
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
            let mut used_rotation = false;
            if can_take_action
                && let Some(rotation) = &timeline.rotation
                && spawn.common.target_id.object_id.is_valid()
            {
                let target = spawn.common.target_id.object_id;
                let situation = Situation {
                    now: Instant::now(),
                    target,
                    target_hp: health_percent.get(&target).copied().unwrap_or(100.0),
                    self_hp: health_percent.get(id).copied().unwrap_or(100.0),
                };
                let mut game_data = game_data.lock();
                let mut mechanics = |action| game_data.get_action_mechanics(action);
                if let Some(action_id) = rotation_state.choose(rotation, &situation, &mut mechanics)
                {
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
                            target_position: target_actor_pos
                                .get(&target)
                                .copied()
                                .unwrap_or(spawn.common.position.0),
                            rotation: spawn.common.rotation,
                            action_id,
                        });
                        let request = ActionRequest {
                            action_id,
                            action_type: ActionType::Action,
                            rotation1: spawn.common.rotation,
                            target: spawn.common.target_id,
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
            if can_take_action
                && !used_rotation
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
    for (id, msg) in &mut actor_moves {
        if let FromServer::ActorMove(_, position, ..) = msg
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
    let begun: Vec<(u32, u32, ObjectId)> = new_action_requests
        .iter()
        .filter_map(|(id, request, _)| {
            let layout_id = instance.find_actor(*id)?.get_common_spawn().layout_id;
            Some((layout_id, request.action_id, *id))
        })
        .collect();
    if let Some(director) = instance.directors.first_mut() {
        for (layout_id, action_id, id) in begun {
            director.on_actor_cast(layout_id, action_id, id);
        }
    }

    for (id, request, cast_time) in new_action_requests {
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
            // Attacks aimed at the ground (e.g. Acorn Armageddon) show their omen where the target stood.
            let aimed_at_ground = {
                let mut game_data = game_data.lock();
                matches!(
                    game_data
                        .get_action_info(request.action_id)
                        .and_then(|info| info.area_shape()),
                    Some(AreaShape::Circle {
                        on_target: true,
                        ..
                    })
                )
            };
            let position;
            {
                let actor = instance.find_actor(id).unwrap();
                position = match actor {
                    NetworkedActor::Npc {
                        cast: Some(cast), ..
                    } if aimed_at_ground => Position(cast.target_position),
                    _ => actor.position(),
                };
            }

            // inform players that this enemy is casting
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
                spell_id: request.action_id as u16,
                action_type: request.action_type,
                omen_delay: 0,
                action_id: request.action_id,
                cast_time: npc_cast_bar_time(cast_time),
                target: request.target.object_id,
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
            if *state == NpcState::Dead
                || *state == NpcState::AoeTest
                || instance.held_npcs.contains(id)
            {
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

/// Has the NPC `actor_id` use `action_id` on `target_id` now, as a director's `use_action` does (and party bots'
/// heals): it stands still while it casts (its area lands where it was aimed now), shows its cast bar, and the action
/// lands when the cast ends.
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
            target_position,
            rotation,
            action_id,
        });
    }

    // Show the cast bar, the same way NPCs cast their own actions.
    if cast_time > 0.0 {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
            spell_id: request.action_id as u16,
            action_type: request.action_type,
            omen_delay: 0,
            action_id: request.action_id,
            cast_time: npc_cast_bar_time(cast_time),
            target: target_id,
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
