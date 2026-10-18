//! Over-time effects (DoTs, HoTs and MP regeneration) driven by status effects.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::Mutex;

use kawari::{
    common::{ObjectId, STRIKING_DUMMY_NAME_ID},
    ipc::zone::ActorControlCategory,
};

use crate::{
    lua::{PeriodicKind, PeriodicSpec},
    server::{actor::update_actor_hp_mp, instance::Instance, network::NetworkState},
};

/// Over-time effects tick every 3 seconds, like retail.
pub const PERIODIC_TICK_INTERVAL: Duration = Duration::from_secs(3);

/// A running over-time effect on an actor.
#[derive(Debug, Clone)]
pub struct PeriodicEffect {
    /// Who has the status effect.
    pub target: ObjectId,
    /// Who applied it.
    pub source: ObjectId,
    /// The status effect driving this. Ticking stops once the target loses it.
    pub effect_id: u16,
    pub kind: PeriodicKind,
    /// Amount applied every tick.
    pub amount: u32,
    /// When the next tick happens.
    pub next_tick: Instant,
}

/// Starts (or refreshes) an over-time effect on `target`.
pub fn add_periodic_effect(
    instance: &mut Instance,
    target: ObjectId,
    source: ObjectId,
    spec: &PeriodicSpec,
) {
    // Re-applying the same effect from the same source refreshes it instead of stacking.
    instance.periodic_effects.retain(|effect| {
        !(effect.target == target && effect.source == source && effect.effect_id == spec.effect_id)
    });

    instance.periodic_effects.push(PeriodicEffect {
        target,
        source,
        effect_id: spec.effect_id,
        kind: spec.kind,
        amount: spec.amount,
        next_tick: Instant::now() + PERIODIC_TICK_INTERVAL,
    });
}

/// An over-time effect's tick: its amount ±5 %, at least 1.
fn vary_tick(amount: u32) -> u32 {
    let variance = 1.0 + (fastrand::f32() * 2.0 - 1.0) * 0.05;
    ((amount as f32 * variance).round() as u32).max(1)
}

/// Applies any over-time effects that are due, and drops ones whose status effect has ended.
pub fn periodic_tick(network: Arc<Mutex<NetworkState>>, instance: &mut Instance) {
    if instance.periodic_effects.is_empty() {
        return;
    }

    let now = Instant::now();
    let mut effects = std::mem::take(&mut instance.periodic_effects);
    let mut actors_to_update = Vec::new();
    let mut heal_enmity = Vec::new();

    effects.retain_mut(|effect| {
        let Some(actor) = instance.find_actor_mut(effect.target) else {
            return false;
        };

        // Stop once the status effect is gone (expired, removed, or the actor died and lost it.)
        let still_active = actor
            .status_effects()
            .map(|effects| effects.get(effect.effect_id).is_some())
            .unwrap_or(false);
        if !still_active {
            return false;
        }

        if actor.get_common_spawn().health_points == 0 {
            return false;
        }

        if now < effect.next_tick {
            return true;
        }
        effect.next_tick += PERIODIC_TICK_INTERVAL;

        // Each tick varies a little like a hit (retail's ticks did; ±5 %.)
        let amount = vary_tick(effect.amount);
        // Damage ticks make enmity for whoever applied them, as retail's do; heal ticks for the healer (below.)
        if effect.kind == PeriodicKind::Damage {
            actor.add_enmity(effect.source, amount);
        }
        let common = actor.get_common_spawn_mut();
        let before = common.health_points;
        match effect.kind {
            PeriodicKind::Damage => {
                if common.name_id != STRIKING_DUMMY_NAME_ID {
                    common.health_points = common.health_points.saturating_sub(amount);
                }
            }
            PeriodicKind::Heal => {
                common.health_points =
                    (common.health_points + amount).min(common.max_health_points);
            }
            PeriodicKind::RestoreMp => {
                let amount = effect.amount.min(u16::MAX as u32) as u16;
                common.resource_points = common
                    .resource_points
                    .saturating_add(amount)
                    .min(common.max_resource_points);
            }
        }

        let restored = common.health_points.saturating_sub(before);

        // Retail (Bob's Chronofoil captures): a damage tick is ActorControl 1541 (0, amount, source, 0xFFFFFFFF), a heal
        // tick 1540 (0, amount, source, critical); a heal tick used to go out as 1540 for both, with the status ID.
        let tick = match effect.kind {
            PeriodicKind::Damage => Some(ActorControlCategory::DamageFlyingText {
                unk1: 0,
                amount,
                source_actor_id: effect.source,
                unk4: u32::MAX,
            }),
            PeriodicKind::Heal => Some(ActorControlCategory::HotDot {
                status_id: 0,
                amount,
                source_actor_id: effect.source,
                critical: 0,
            }),
            PeriodicKind::RestoreMp => None,
        };
        if effect.kind == PeriodicKind::Heal {
            heal_enmity.push((effect.source, restored));
        }
        if let Some(tick) = tick {
            let mut network = network.lock();
            network.send_ac_in_range_inclusive_instance(instance, effect.target, tick);
        }

        if !actors_to_update.contains(&effect.target) {
            actors_to_update.push(effect.target);
        }

        true
    });

    // Keep any effects added while we were ticking.
    effects.append(&mut instance.periodic_effects);
    instance.periodic_effects = effects;

    for actor_id in actors_to_update {
        update_actor_hp_mp(network.clone(), instance, actor_id);
    }
    for (healer, effective) in heal_enmity {
        crate::server::action::add_healing_enmity(instance, healer, effective);
    }
}
