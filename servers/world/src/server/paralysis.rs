//! Paralysis (status 17), as retail plays it out (Bob's Chronofoil captures, 85 windows of the status on players and
//! NPCs, 2026-10-14 to 2026-11-01): while an actor has it, every 3 s it may seize up (about one roll in two: 290 procs
//! in 593 three-second ticks). A proc shows the paralysis over the actor (ActorControl 23, `17, 30, 0, actor`, to
//! everyone around) and cancels what it's casting: its spell (CancelCast 541), anything else (540) or a quest's
//! progress bar (540, kind 4). Nothing it did came sooner than a second after a proc (NPCs' next action 0.98 s or more
//! later, players' next ActionRequest 1.03-1.26 s later), so NPCs don't act for a second after one; the client keeps
//! the player from acting itself.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::Mutex;

use kawari::{
    common::{ActionCategory, ObjectId},
    ipc::zone::ActorControlCategory,
};

use crate::{
    GameData,
    server::{
        actor::NetworkedActor,
        instance::{EventActionInterruption, Instance},
        network::NetworkState,
    },
};

/// The Status sheet's Paralysis.
pub const STATUS_PARALYSIS: u16 = 17;
/// How often a paralysed actor may seize up (retail's procs were always a multiple of 3 s apart.)
const ROLL_INTERVAL: Duration = Duration::from_secs(3);
/// The chance it does at each roll.
const PROC_CHANCE: f32 = 0.5;
/// How long a paralysed NPC can't act after seizing up.
pub const LOCKOUT: Duration = Duration::from_secs(1);
/// CancelCast's first value for a spell cut short by paralysis, and for anything else (an NPC's weaponskill or
/// ability, a mount, a progress bar.)
const CANCEL_SPELL: u32 = 541;
const CANCEL_OTHER: u32 = 540;

/// Who is paralysed: when each next rolls, and until when an NPC that seized up can't act.
#[derive(Debug, Default)]
pub struct Paralysis {
    next_roll: HashMap<ObjectId, Instant>,
    locked_until: HashMap<ObjectId, Instant>,
}

impl Paralysis {
    /// Whether this actor seized up less than a second ago.
    pub fn locked(&self, actor_id: ObjectId, now: Instant) -> bool {
        self.locked_until
            .get(&actor_id)
            .is_some_and(|until| *until > now)
    }
}

/// Rolls for everyone paralysed whose 3 s are up.
pub fn paralysis_tick(
    network: Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
) {
    let now = Instant::now();
    instance
        .paralysis
        .locked_until
        .retain(|_, until| *until > now);

    let paralysed: Vec<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| {
            actor.get_common_spawn().health_points > 0
                && actor
                    .status_effects()
                    .is_some_and(|effects| effects.get(STATUS_PARALYSIS).is_some())
        })
        .map(|(id, _)| *id)
        .collect();
    instance
        .paralysis
        .next_roll
        .retain(|id, _| paralysed.contains(id));
    if paralysed.is_empty() {
        return;
    }

    let mut seized = Vec::new();
    for id in paralysed {
        let next = instance
            .paralysis
            .next_roll
            .entry(id)
            .or_insert(now + ROLL_INTERVAL);
        if now < *next {
            continue;
        }
        *next += ROLL_INTERVAL;
        if fastrand::f32() < PROC_CHANCE {
            seized.push(id);
        }
    }

    for id in seized {
        seize_up(network.clone(), game_data, instance, id, now);
    }
}

/// One proc: shown to everyone around, and whatever it was casting is cancelled.
fn seize_up(
    network: Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
    actor_id: ObjectId,
    now: Instant,
) {
    if matches!(
        instance.find_actor(actor_id),
        Some(NetworkedActor::Npc { .. })
    ) {
        instance
            .paralysis
            .locked_until
            .insert(actor_id, now + LOCKOUT);
    }

    let cancelled = instance.cancel_actor_casts(actor_id);
    if let Some(NetworkedActor::Npc { cast, .. }) = instance.find_actor_mut(actor_id) {
        *cast = None;
    }
    let categories: Vec<(u32, Option<ActionCategory>)> = {
        let mut game_data = game_data.lock();
        cancelled
            .into_iter()
            .map(|action_id| {
                let category = game_data
                    .get_casttime(action_id)
                    .map(|(category, _)| category);
                (action_id, category)
            })
            .collect()
    };

    {
        let mut network = network.lock();
        // Retail's values as they were: what each means isn't known.
        network.send_ac_in_range_inclusive_instance(
            instance,
            actor_id,
            ActorControlCategory::FloatingNumberVFX {
                unk1: STATUS_PARALYSIS as u32,
                kind: 30,
                amount: 0,
                object_id: actor_id,
            },
        );
        for (action_id, category) in categories {
            let unk1 = if category == Some(ActionCategory::Spell) {
                CANCEL_SPELL
            } else {
                CANCEL_OTHER
            };
            network.send_ac_in_range_inclusive_instance(
                instance,
                actor_id,
                ActorControlCategory::CancelCast {
                    unk1,
                    unk2: 1,
                    action_id,
                    unk3: 1,
                },
            );
        }
        instance.interrupt_event_action(&mut network, actor_id, EventActionInterruption::Paralysed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_for_a_second() {
        let mut paralysis = Paralysis::default();
        let id = ObjectId(0x4000_0001);
        let now = Instant::now();
        assert!(!paralysis.locked(id, now));
        paralysis.locked_until.insert(id, now + LOCKOUT);
        assert!(paralysis.locked(id, now + Duration::from_millis(990)));
        assert!(!paralysis.locked(id, now + LOCKOUT));
    }
}
