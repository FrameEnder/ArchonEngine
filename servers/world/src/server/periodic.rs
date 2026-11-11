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

    // A target's over-time effects all tick together, as retail's did (one tick for all its HoTs, one for all its
    // DoTs, below): a new one joins the target's ticks when it has any.
    let next_tick = instance
        .periodic_effects
        .iter()
        .find(|effect| effect.target == target)
        .map_or(Instant::now() + PERIODIC_TICK_INTERVAL, |effect| {
            effect.next_tick
        });

    instance.periodic_effects.push(PeriodicEffect {
        target,
        source,
        effect_id: spec.effect_id,
        kind: spec.kind,
        amount: spec.amount,
        next_tick,
    });
}

/// What a target's ticks of one kind add up to, sent as one (`tick_control`.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TickTotal {
    pub target: ObjectId,
    pub kind: PeriodicKind,
    pub amount: u32,
    /// The first of its effects' sources (retail named one HoT's healer even with two healers' HoTs on the target.)
    pub source: ObjectId,
}

/// Adds a tick to its target's total of that kind.
pub(crate) fn add_to_totals(
    totals: &mut Vec<TickTotal>,
    target: ObjectId,
    kind: PeriodicKind,
    amount: u32,
    source: ObjectId,
) {
    match totals
        .iter_mut()
        .find(|total| total.target == target && total.kind == kind)
    {
        Some(total) => total.amount += amount,
        None => totals.push(TickTotal {
            target,
            kind,
            amount,
            source,
        }),
    }
}

/// A target's tick of one kind as retail sent it (Bob's Chronofoil captures): all of its DoTs as ActorControl 1541
/// `(0, amount, source, 0xFFFFFFFF)`, all of its HoTs as 1540 `(0, amount, source, critical)`, and MP restored over
/// time (Lucid Dreaming) as 23 `(0, 11, amount, the target)`, 785 of them, 550 each.
pub(crate) fn tick_control(total: &TickTotal) -> ActorControlCategory {
    match total.kind {
        PeriodicKind::Damage => ActorControlCategory::DamageFlyingText {
            unk1: 0,
            amount: total.amount,
            source_actor_id: total.source,
            unk4: u32::MAX,
        },
        PeriodicKind::Heal => ActorControlCategory::HotDot {
            status_id: 0,
            amount: total.amount,
            source_actor_id: total.source,
            critical: 0,
        },
        PeriodicKind::RestoreMp => ActorControlCategory::FloatingNumberVFX {
            unk1: 0,
            kind: MP_RESTORED,
            amount: total.amount,
            object_id: total.target,
        },
    }
}

/// ActorControl 23's kind for MP restored (a TargetEffectKind.)
const MP_RESTORED: u32 = 11;

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
    let mut totals = Vec::new();
    let mut broken = Vec::new();

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
        // A barrier takes a damage tick first (`server/barrier.rs`); one used up loses its status, below.
        let through = if effect.kind == PeriodicKind::Damage {
            let (through, used_up) =
                crate::server::barrier::absorb(instance, effect.target, amount);
            broken.extend(used_up.into_iter().map(|status| (effect.target, status)));
            through
        } else {
            amount
        };
        let Some(actor) = instance.find_actor_mut(effect.target) else {
            return false;
        };
        let common = actor.get_common_spawn_mut();
        let before = common.health_points;
        match effect.kind {
            PeriodicKind::Damage => {
                if common.name_id != STRIKING_DUMMY_NAME_ID {
                    common.health_points = common.health_points.saturating_sub(through);
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

        // Shown as one tick per target and kind (`tick_control`), once everything due has ticked.
        let shown = if effect.kind == PeriodicKind::RestoreMp {
            effect.amount.min(u16::MAX as u32)
        } else {
            amount
        };
        add_to_totals(
            &mut totals,
            effect.target,
            effect.kind,
            shown,
            effect.source,
        );
        if effect.kind == PeriodicKind::Heal {
            heal_enmity.push((effect.source, restored));
        }

        if !actors_to_update.contains(&effect.target) {
            actors_to_update.push(effect.target);
        }

        true
    });

    // Keep any effects added while we were ticking.
    effects.append(&mut instance.periodic_effects);
    instance.periodic_effects = effects;

    for (target, status) in broken {
        crate::server::effect::lose_effect_instance(network.clone(), instance, target, status);
    }

    if !totals.is_empty() {
        let mut network = network.lock();
        for total in &totals {
            network.send_ac_in_range_inclusive_instance(
                instance,
                total.target,
                tick_control(total),
            );
        }
    }

    for actor_id in actors_to_update {
        update_actor_hp_mp(network.clone(), instance, actor_id);
    }
    for (healer, effective) in heal_enmity {
        crate::server::action::add_healing_enmity(instance, healer, effective);
    }
}
