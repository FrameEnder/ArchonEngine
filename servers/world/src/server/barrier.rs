//! Barriers: HP a status absorbs before the one who has it is hurt (Galvanize, Catalyze...), as retail plays them.
//!
//! Retail's EffectResult carries what's left of a target's barriers as a share of their maximum HP (its `shield` byte,
//! in percent): 7,270 results in Bob's Chronofoil captures, with statuses like Galvanize (297). A script gives a
//! barrier with the status that holds it (`effects:barrier(status, amount)`); damage takes from the barriers first,
//! and one used up loses its status.

use std::{collections::HashMap, sync::Arc};

use kawari::common::ObjectId;
use parking_lot::Mutex;

use crate::server::{
    actor::NetworkedActor, effect::lose_effect_instance, instance::Instance, network::NetworkState,
};

/// One barrier: the status holding it and the HP it has left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Barrier {
    pub status: u16,
    pub remaining: u32,
}

/// Every actor's barriers.
pub type Barriers = HashMap<ObjectId, Vec<Barrier>>;

/// `target` gets a barrier of `amount` held by `status` (another from the same status replaces it.)
pub fn grant(instance: &mut Instance, target: ObjectId, status: u16, amount: u32) {
    let barriers = instance.barriers.entry(target).or_default();
    barriers.retain(|barrier| barrier.status != status);
    if amount > 0 {
        barriers.push(Barrier {
            status,
            remaining: amount,
        });
    }
}

/// `target` gets the barrier an action gives (`EffectsBuilder::barriers`), its size worked out on them.
pub fn grant_spec(instance: &mut Instance, target: ObjectId, spec: &crate::lua::BarrierSpec) {
    let amount = match spec.size {
        crate::lua::BarrierSize::Hp(amount) => amount,
        crate::lua::BarrierSize::PercentOfMaxHp(percent) => instance
            .find_actor(target)
            .map(|actor| actor.get_common_spawn().max_health_points * percent as u32 / 100)
            .unwrap_or(0),
    };
    grant(instance, target, spec.status, amount);
}

/// Drops `target`'s barriers whose status is gone (expired, removed, or they died.)
fn prune(instance: &mut Instance, target: ObjectId) {
    let statuses: Vec<u16> = instance
        .find_actor(target)
        .and_then(NetworkedActor::status_effects)
        .map(|effects| effects.ids().collect())
        .unwrap_or_default();
    if let Some(barriers) = instance.barriers.get_mut(&target) {
        barriers.retain(|barrier| statuses.contains(&barrier.status));
        if barriers.is_empty() {
            instance.barriers.remove(&target);
        }
    }
}

/// `damage` meets `target`'s barriers, oldest first: what gets through to their HP, and the statuses whose barrier was
/// used up (to be taken off them.)
pub fn absorb(instance: &mut Instance, target: ObjectId, damage: u32) -> (u32, Vec<u16>) {
    prune(instance, target);
    let Some(barriers) = instance.barriers.get_mut(&target) else {
        return (damage, Vec::new());
    };
    let mut left = damage;
    let mut used_up = Vec::new();
    for barrier in barriers.iter_mut() {
        if left == 0 {
            break;
        }
        let taken = barrier.remaining.min(left);
        barrier.remaining -= taken;
        left -= taken;
        if barrier.remaining == 0 {
            used_up.push(barrier.status);
        }
    }
    barriers.retain(|barrier| barrier.remaining > 0);
    (left, used_up)
}

/// `absorb`, and the statuses of the barriers it used up taken off `target` (LoseEffect and the status list): what
/// gets through to their HP.
pub fn absorb_and_break(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    target: ObjectId,
    damage: u32,
) -> u32 {
    let (through, used_up) = absorb(instance, target, damage);
    for status in used_up {
        lose_effect_instance(network.clone(), instance, target, status);
    }
    through
}

/// What's left of `target`'s barriers as a share of their maximum HP, in percent (EffectResult's `shield`.)
pub fn share(instance: &mut Instance, target: ObjectId) -> u8 {
    prune(instance, target);
    let total: u64 = instance
        .barriers
        .get(&target)
        .map(|barriers| {
            barriers
                .iter()
                .map(|barrier| barrier.remaining as u64)
                .sum()
        })
        .unwrap_or(0);
    let max = instance
        .find_actor(target)
        .map(|actor| actor.get_common_spawn().max_health_points as u64)
        .unwrap_or(0);
    if total == 0 || max == 0 {
        return 0;
    }
    ((total * 100).div_ceil(max)).min(100) as u8
}
