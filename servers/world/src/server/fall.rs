//! Fall damage, as retail deals it (Bob's Chronofoil captures: 49 falls that hurt and 470 landings that didn't, levels
//! 6 to 88). The client says when the player leaves the ground (UpdatePositionHandler's MoveAnimationState
//! LeavingCollision) and lands (EnteringCollision); the drop is how far below the last spot they stood on they landed.
//! Ten yalms or more hurts (9.99 never did, 10.0 did), by the player's maximum HP: 10 % at 10 yalms and 0.228 % more
//! for each square yalm past it (`10 + 0.228 × (drop − 10)²` %, within 1.2 % of all 19 falls whose height was
//! measured, 4 % at most), and out of combat never below 1 HP (retail's 272 of 273 and 33,381 of 33,382 after the
//! highest falls); in combat a fall can kill (a long-time player's note, Bob, 2026-10-28).
//! Retail showed it with ActorControl 45 (the damage, 1) from the player, then their StatusEffectList.
//! A mounted player is hurt as one on foot is (since 2026-11-18): Connor's capture e834e436 has one, mounted (mode 4),
//! down 12 yalms for 241, and no capture has a mounted drop of 10 yalms that didn't hurt; Kawari spared the mounted,
//! and Bob rode off a cliff in the Central Shroud, 39.6 yalms, for nothing ("no fall damage from cliff".) What the
//! client sends in flight wasn't seen, so a mounted drop that took much longer than a fall of its height would
//! (`fell_freely`) is taken for a flight and spared.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use kawari::{
    common::{CharacterMode, JumpState, MoveAnimationState, ObjectId, Position},
    ipc::zone::{ActorControlCategory, Condition},
};

use crate::{
    FromServer,
    server::{
        actor::{NetworkedActor, kill_actor},
        effect::send_effects_list,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
    },
};

/// How far a fall has to drop to hurt, in yalms.
pub const FALL_DAMAGE_FROM: f32 = 10.0;
/// What a fall of `FALL_DAMAGE_FROM` takes, in percent of the maximum HP, and how much more each square yalm past it.
const FALL_DAMAGE_BASE_PERCENT: f32 = 10.0;
const FALL_DAMAGE_PER_SQUARE_YALM: f32 = 0.228;
/// ActorControl 45's second value (retail's, every fall but two at level 87.)
const FALL_DAMAGE_UNK: u32 = 1;

/// How fast a fall gathers speed, in yalms a second each second: Bob's 39.3 yalms off the Central Shroud's cliff took
/// 2.05 s between the client's leaving the ground and landing (the server's capture of 2026-11-18.)
const FALL_GRAVITY: f32 = 18.7;
/// A mounted drop that took longer than this many times a fall of its height, and this much more, was a flight (a
/// guess: no flight has been looked at.)
const FLIGHT_TIMES_A_FALL: f32 = 1.6;
const FLIGHT_SLACK: f32 = 0.6;

/// Where a player stood last and, while they're in the air, where they left the ground from and when.
#[derive(Debug, Default, Clone, Copy)]
pub struct FallState {
    grounded_y: Option<f32>,
    from_y: Option<f32>,
    left_at: Option<Instant>,
}

/// Whether a drop of `drop` yalms that took `airborne` was a fall (and not a mount flying down.)
pub fn fell_freely(drop: f32, airborne: Duration) -> bool {
    if drop <= 0.0 {
        return true;
    }
    let fall = (2.0 * drop / FALL_GRAVITY).sqrt();
    airborne.as_secs_f32() <= fall * FLIGHT_TIMES_A_FALL + FLIGHT_SLACK
}

/// Whether a landing hurts at all: not the dead, not a gimmick's jump (its own landing), and a mounted player only
/// when they fell (`fell_freely`.)
pub fn landing_hurts(
    mounted: bool,
    dead: bool,
    gimmick_jump: bool,
    drop: f32,
    airborne: Duration,
) -> bool {
    !dead && !gimmick_jump && (!mounted || fell_freely(drop, airborne))
}

/// What a fall of `drop` yalms takes from someone with `max_hp` and `hp` now (0: nothing): out of combat it leaves
/// them 1 HP at least, in combat it can take everything.
pub fn fall_damage(drop: f32, max_hp: u32, hp: u32, in_combat: bool) -> u32 {
    if drop < FALL_DAMAGE_FROM || hp == 0 {
        return 0;
    }
    let past = drop - FALL_DAMAGE_FROM;
    let percent = FALL_DAMAGE_BASE_PERCENT + FALL_DAMAGE_PER_SQUARE_YALM * past * past;
    let damage = (max_hp as f32 * percent / 100.0).round() as u32;
    if in_combat {
        damage.min(hp)
    } else {
        damage.min(hp - 1)
    }
}

/// A player moved: follows their jumps and falls, and hurts them when they land far enough below where they left.
pub fn player_moved(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    actor_id: ObjectId,
    position: Position,
    anim_state: MoveAnimationState,
    jump_state: JumpState,
) {
    let y = position.0.y;
    let state = instance.falls.entry(actor_id).or_default();
    match anim_state {
        MoveAnimationState::LeavingCollision if state.from_y.is_none() => {
            state.from_y = Some(state.grounded_y.unwrap_or(y));
            state.left_at = Some(Instant::now());
            return;
        }
        MoveAnimationState::EnteringCollision => {
            let from = state.from_y.take();
            let airborne = state
                .left_at
                .take()
                .map(|left| left.elapsed())
                .unwrap_or_default();
            state.grounded_y = Some(y);
            if let Some(from) = from {
                hurt(network, instance, actor_id, from - y, airborne);
            }
            return;
        }
        _ => {}
    }
    if state.from_y.is_none()
        && anim_state == MoveAnimationState::None
        && jump_state == JumpState::NoneOrFalling
    {
        state.grounded_y = Some(y);
    }
}

fn hurt(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    actor_id: ObjectId,
    drop: f32,
    airborne: Duration,
) {
    let Some(NetworkedActor::Player {
        spawn,
        executing_gimmick_jump,
        conditions,
        ..
    }) = instance.find_actor_mut(actor_id)
    else {
        return;
    };
    let common = &mut spawn.common;
    // Already down, a gimmick's jump (its own landing) or a mount flying down: nothing happens.
    if !landing_hurts(
        common.mode == CharacterMode::Mounted,
        common.mode == CharacterMode::Dead,
        *executing_gimmick_jump,
        drop,
        airborne,
    ) {
        return;
    }
    let in_combat = conditions.has_condition(Condition::InCombat);
    let damage = fall_damage(
        drop,
        common.max_health_points,
        common.health_points,
        in_combat,
    );
    if damage == 0 {
        return;
    }
    common.health_points -= damage;
    tracing::info!("{actor_id} fell {drop:.1} yalms and took {damage}");
    network.lock().send_in_range_inclusive_instance(
        actor_id,
        instance,
        FromServer::ActorControl(
            actor_id,
            ActorControlCategory::FallDamage {
                damage,
                unk: FALL_DAMAGE_UNK,
            },
        ),
        DestinationNetwork::ZoneClients,
    );
    send_effects_list(network.clone(), instance, actor_id);
    if instance
        .find_actor(actor_id)
        .is_some_and(|actor| actor.get_common_spawn().health_points == 0)
    {
        kill_actor(network.clone(), instance, actor_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_retails_falls() {
        // Bob's captures: 9.99 yalms nothing, 10.0 yalms 3511 of 35118.
        assert_eq!(fall_damage(9.99, 35118, 35118, false), 0);
        assert_eq!(fall_damage(10.0, 35118, 35118, false), 3512);
        // 20.58 yalms: 250 of 715 on retail.
        assert!((fall_damage(20.58, 715, 715, false) as i32 - 250).abs() <= 10);
        // 28.87 yalms: 245 of 273.
        assert!((fall_damage(28.87, 273, 273, false) as i32 - 245).abs() <= 6);
        // Out of combat never below 1 HP; in combat it can kill.
        assert_eq!(fall_damage(40.0, 273, 273, false), 272);
        assert_eq!(fall_damage(40.0, 273, 1, false), 0);
        assert_eq!(fall_damage(40.0, 273, 273, true), 273);
    }

    /// Bob, 2026-11-18 ("no fall damage from cliff"): he rode off a cliff in the Central Shroud, 39.6 yalms down in
    /// 2.05 s, and took nothing, because the mounted were spared. Retail hurts them (Connor's capture e834e436: 241
    /// for 12 yalms, mounted.) A mount that took far longer than a fall would is flying, and isn't hurt.
    #[test]
    fn a_mounted_fall_hurts() {
        let fall = Duration::from_secs_f32(2.05);
        assert!(landing_hurts(true, false, false, 39.6, fall));
        assert!(landing_hurts(false, false, false, 39.6, fall));
        assert_eq!(fall_damage(39.6, 2242, 2242, false), 2241);
        // The dead and a gimmick's jump, as before.
        assert!(!landing_hurts(false, true, false, 39.6, fall));
        assert!(!landing_hurts(true, false, true, 39.6, fall));
        // Flying down the same height in ten seconds; on foot nobody flies.
        assert!(!landing_hurts(
            true,
            false,
            false,
            39.6,
            Duration::from_secs(10)
        ));
        assert!(landing_hurts(
            false,
            false,
            false,
            39.6,
            Duration::from_secs(10)
        ));
        // Connor's: 12 yalms is a fall of 1.1 s.
        assert!(fell_freely(12.0, Duration::from_secs_f32(1.2)));
        assert!(!fell_freely(12.0, Duration::from_secs_f32(4.0)));
    }
}
