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
    FromServer, GameData,
    server::{
        actor::NetworkedActor,
        instance::{EventActionInterruption, Instance},
        network::{DestinationNetwork, NetworkState},
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

/// Who is paralysed: when each next rolls, until when an NPC that seized up can't act, and when a player that seized
/// up is let go (`release_players`.)
#[derive(Debug, Default)]
pub struct Paralysis {
    next_roll: HashMap<ObjectId, Instant>,
    locked_until: HashMap<ObjectId, Instant>,
    release_at: HashMap<ObjectId, Instant>,
}

impl Paralysis {
    /// Whether this actor seized up less than a second ago.
    pub fn locked(&self, actor_id: ObjectId, now: Instant) -> bool {
        self.locked_until
            .get(&actor_id)
            .is_some_and(|until| *until > now)
    }
}

/// The living players and NPCs with paralysis. Only those: objects and treasure coffers have no health
/// (`get_common_spawn` panics on them: the North Shroud's gathering points brought the world server down, Bob's bundle
/// 20261101-211113.)
fn paralysed_actors(instance: &Instance) -> Vec<ObjectId> {
    instance
        .actors
        .iter()
        .filter(|(_, actor)| {
            matches!(
                actor,
                NetworkedActor::Player { .. } | NetworkedActor::Npc { .. }
            ) && actor.get_common_spawn().health_points > 0
                && actor
                    .status_effects()
                    .is_some_and(|effects| effects.get(STATUS_PARALYSIS).is_some())
        })
        .map(|(id, _)| *id)
        .collect()
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
    release_players(&network, instance, now);

    let paralysed = paralysed_actors(instance);
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

/// A player's client seizes up on its own at a proc (the FloatingNumberVFX) and stays that way until the server next
/// sends its conditions: retail sent them, unchanged, 0.98 s after every proc on the player (Bob's Chronofoil capture
/// 44df02fc, all of them), which is why its next ActionRequest came 1.03-1.26 s after. Kawari never did: a proc that
/// came while the player was casting got the Casting condition's own updates soon after and passed, but one that cut
/// a cast short (the conditions sent just before it, then nothing) left the player unable to act or move for good,
/// the status long gone (Bob's bundles 20261104-150457 to -180355.) Sent at the first tick a second after the proc
/// (procs fall on ticks, so that's 1.0 s), whether or not the player is still paralysed.
fn release_players(network: &Arc<Mutex<NetworkState>>, instance: &mut Instance, now: Instant) {
    // A tick's slack, so a release due a hair after this tick goes now rather than half a second late.
    let due = now + Duration::from_millis(50);
    let released: Vec<ObjectId> = instance
        .paralysis
        .release_at
        .iter()
        .filter(|(_, at)| **at <= due)
        .map(|(id, _)| *id)
        .collect();
    if released.is_empty() {
        return;
    }
    let mut network = network.lock();
    for id in released {
        instance.paralysis.release_at.remove(&id);
        network.send_to_by_actor_id(
            id,
            FromServer::ParalysisRecovered,
            DestinationNetwork::ZoneClients,
        );
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

    let cancels: Vec<(u32, u32)> = categories
        .into_iter()
        .map(|(action_id, category)| {
            let unk1 = if category == Some(ActionCategory::Spell) {
                CANCEL_SPELL
            } else {
                CANCEL_OTHER
            };
            (unk1, action_id)
        })
        .collect();
    let is_player = matches!(
        instance.find_actor(actor_id),
        Some(NetworkedActor::Player { .. })
    );
    if is_player {
        instance
            .paralysis
            .release_at
            .insert(actor_id, now + LOCKOUT);
    }

    {
        let mut network = network.lock();
        // Retail's values as they were: what each means isn't known. Everyone around gets the CancelCasts and the proc
        // as ActorControl from the actor; the player's own come from their connection (`paralysis_proc`), with the
        // conditions first when a cast was cut short, as retail.
        for (unk1, action_id) in &cancels {
            network.send_ac_in_range_instance(
                instance,
                actor_id,
                ActorControlCategory::CancelCast {
                    unk1: *unk1,
                    unk2: 1,
                    action_id: *action_id,
                    unk3: 1,
                },
            );
        }
        network.send_ac_in_range_instance(
            instance,
            actor_id,
            ActorControlCategory::FloatingNumberVFX {
                unk1: STATUS_PARALYSIS as u32,
                kind: 30,
                amount: 0,
                object_id: actor_id,
            },
        );
        if is_player {
            network.send_to_by_actor_id(
                actor_id,
                FromServer::ParalysisProc(cancels),
                DestinationNetwork::ZoneClients,
            );
        }
        instance.interrupt_event_action(&mut network, actor_id, EventActionInterruption::Paralysed);
    }

    // Then StatusEffectList and UpdateClassInfo, as the player had them right after every proc on retail (Bob's
    // Chronofoil capture 44df02fc). They're retail's 3 s tick, which the procs fall on, rather than the proc's own.
    if is_player {
        crate::server::effect::send_effects_list(network.clone(), instance, actor_id);
        network.lock().send_to_by_actor_id(
            actor_id,
            FromServer::SendClassInfo,
            DestinationNetwork::ZoneClients,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kawari::ipc::zone::SpawnObject;

    #[test]
    fn objects_are_left_alone() {
        // A gathering point (an object) in the instance: no panic, nobody paralysed.
        let mut instance = Instance::default();
        instance.actors.insert(
            ObjectId(0x4000_0002),
            NetworkedActor::Object {
                object: SpawnObject::default(),
                layer_name: String::new(),
            },
        );
        assert!(paralysed_actors(&instance).is_empty());
    }

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
