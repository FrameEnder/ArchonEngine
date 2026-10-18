use std::{
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    sync::Arc,
    time::{Duration, Instant},
};

use glam::Vec3A;
use kawari::{
    common::{
        AUTO_ATTACK_RATE, JumpState, MOB_WANDER_TIME, MoveAnimationState, MoveAnimationType,
        ObjectId, ObjectTypeId, ObjectTypeKind, Position, STRIKING_DUMMY_NAME_ID, TimepointData,
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
        actor::{
            NetworkedActor, NpcCast, NpcState, NpcTarget, set_shared_group_timeline_state,
            update_actor_hp_mp,
        },
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

/// How close (in yalms) a player has to come for an aggressive enemy to notice them.
/// TODO: per enemy, from Kawari Recorder's "World" recording (attacks on sight, from how far.)
const AGGRO_RANGE: f32 = 15.0;

/// Aggressive enemies ignore players this many levels or more above them.
const AGGRO_LEVEL_GAP: u8 = 7;

/// How far (in yalms) an enemy will chase from where it spawned before giving up, going home and healing up.
/// TODO: per enemy, from Kawari Recorder's "World" recording (gave up at ~N yalms from spawn.)
const LEASH_DISTANCE: f32 = 40.0;

/// How close to home an enemy that gave up has to get before it pays attention again.
const HOME_DISTANCE: f32 = 3.0;

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
    let mut healed = Vec::new();
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
            let (position, rotation) =
                if distance <= min_distance && !(strayed && !current_path.is_empty()) {
                    (
                        Some(spawn.common.position),
                        Some(rotate(spawn.common.position.0, target_pos)),
                    )
                } else if !current_path.is_empty() {
                    // otherwise, Follow current path
                    let next_position = current_path[0];

                    let current_position = last_position.unwrap_or(spawn.common.position.0);

                    (
                        Some(Position(Vec3A::lerp(
                            current_position,
                            next_position,
                            *current_path_lerp,
                        ))),
                        Some(rotate(current_position, next_position)),
                    )
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
                actor_moves.push((
                    *id,
                    FromServer::ActorMove(
                        *id,
                        position,
                        rotation,
                        MoveAnimationType::empty(),
                        MoveAnimationState::None,
                        JumpState::NoneOrFalling,
                    ),
                ));
            }
        }
    }

    let mut newly_acquired_targets = Vec::new();
    let mut new_action_requests = Vec::new();
    let mut new_timeline_states = Vec::new();
    let mut dropped_targets = Vec::new();

    // mut pass
    for (id, actor) in &mut instance.actors {
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
            // also when the one it's after is that far from its home: an enemy that can't find a path (no navmesh
            // route, so it never moves) would otherwise keep them in combat forever, wherever they went in the zone.
            let from_home = Vec3A::distance(spawn.common.position.0, *home);
            let target_from_home = match current_target {
                Some(NpcTarget::Actor(target)) => enemies
                    .iter()
                    .find(|(id, _, _)| id == target)
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
                spawn.common.health_points = spawn.common.max_health_points;
                healed.push(*id);
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

            // Go after whoever we hate the most (that's still around), sticking with our current target on a tie.
            enmity.retain(|actor, _| enemies.iter().any(|(id, _, _)| id == actor));
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
            // they're far above them.
            if *state == NpcState::Wander
                && !*returning_home
                && spawn
                    .character_data_flags
                    .contains(CharacterDataFlag::HOSTILE)
            {
                let mut game_data = game_data.lock();
                let possible_enemies =
                    game_data.get_battalion_enemies(spawn.common.battalion as u32);

                for (target_id, position, battalion) in &enemies {
                    if !possible_enemies[*battalion as usize] {
                        continue;
                    }
                    if levels.get(target_id).is_some_and(|level| {
                        *level >= spawn.common.level.saturating_add(AGGRO_LEVEL_GAP)
                    }) {
                        continue;
                    }

                    if Vec3A::distance(position.0, spawn.common.position.0) < AGGRO_RANGE {
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
                    && Instant::now().duration_since(*last_wander_timestamp) > MOB_WANDER_TIME
                {
                    // Wander around home, rather than drifting further away each time.
                    *current_target = Some(NpcTarget::Position(
                        instance.navmesh.find_wander_position(*home),
                    ));
                    *last_wander_timestamp = Instant::now();
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

                *current_path_lerp = f32::clamp(*current_path_lerp + (2.0 / distance), 0.0, 1.0);
            }

            let mut reset_target = false;
            let mut reset_target_for_path = false;
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
                    if needs_repath {
                        let goal = strayed_to.unwrap_or(target_pos);
                        let mut path: VecDeque<Vec3A> = instance
                            .navmesh
                            .calculate_path(current_pos, goal, max_off_mesh_gap)
                            .into();
                        // In content no path at all (e.g. starting off the navmesh, among a tree's roots) means walking
                        // straight there: dropping the target there sent To Guard a Guardian's second wave wandering
                        // back to where they appeared, around the tree (2026-10-17 bundle).
                        if path.is_empty() && in_content {
                            path.push_back(goal);
                        }

                        *current_path = path.clone();
                        // Start the new path from where we actually are.
                        *last_position = Some(current_pos);
                        *current_path_lerp = 0.0;

                        // Drop the current target if we can't path to them too
                        if path.is_empty() {
                            reset_target = true;
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

            // Only update the timeline on exact second marks
            if can_take_action && (*timeline_position % 2) == 0 {
                // TODO: something worth thinking about is whether to simplify timeline_always_play, and have it always play anyway but skip Action points?

                // NOTE: the "+ 0.5" is a hack to ensure the last timepoint is always counted
                let timeline_position_seconds = *timeline_position / 2;
                let real_timeline_position =
                    timeline_position_seconds as f32 % (timeline.duration() as f32 + 0.5);
                for timepoint in timeline.points_at(real_timeline_position as i32) {
                    match &timepoint.data {
                        TimepointData::Action { action_id, .. } => {
                            if spawn.common.target_id.object_id.is_valid() && can_take_action {
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
            }

            if reset_target_for_path {
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

    // inform clients of the NPCs new positions
    for (id, msg) in actor_moves {
        let mut network = network.lock();
        network.send_in_range_instance(id, instance, msg, DestinationNetwork::ZoneClients);
    }

    for (id, request, cast_time) in new_action_requests {
        // The client drops NPCs out of their battle stance after they act, so draw their weapon again half a second
        // after this lands.
        instance.insert_task(
            ClientId::default(),
            id,
            Duration::from_secs_f32(cast_time + 0.5),
            QueuedTaskData::DrawWeapon,
        );

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
                cast_time,
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

    // Those that gave up a chase are back to full health, and nobody claims them any more.
    for id in healed {
        update_actor_hp_mp(network.clone(), instance, id);
        first_attack::clear(&mut network.lock(), instance, id);
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
            ..
        } = actor
        {
            if *state == NpcState::Dead || *state == NpcState::AoeTest {
                continue;
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
                        director.on_actor_aggro(spawn.common.layout_id);
                    }
                }

                haters.entry(*actor).or_default();
                haters.get_mut(actor).unwrap().push(*id);
            }
        }
    }
}
