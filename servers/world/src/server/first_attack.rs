//! Who first attacked an actor: its "claim" (FirstAttack, opcode 374 on 7.57).
//!
//! From Bob's Chronofoil captures of retail (2026-10-14 to 17, about 1,100 of these in the Central Shroud and
//! Thanalan):
//! * the first time an actor takes a hostile action from someone, retail tells everyone nearby who it was, with the
//!   attacked actor as the packet's source, just before that action's EffectResult: an enemy hit by a player (or by an
//!   allied NPC), and a player hit by an enemy;
//! * it stays with the first attacker; a later attacker only takes over once the first is gone (dead or left);
//! * a player's is cleared (kind 0, no ID) after their fight is over: 0 to 3 s after their enemy list empties on
//!   retail; Kawari clears it as their InCombat condition goes;
//! * an enemy's is cleared when it gives up a chase and walks home;
//! * the same ID is CommonSpawn's `combat_tagger_id`, so whoever sees the actor spawn later knows it too.

use kawari::{
    common::{ObjectId, ObjectTypeId},
    ipc::zone::{ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    FromServer,
    server::{
        actor::NetworkedActor,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
    },
};

/// FirstAttack's `kind`: tagged by someone, or cleared.
const TAGGED: u32 = 1;
const CLEARED: u32 = 0;

fn send(
    network: &mut NetworkState,
    instance: &Instance,
    actor_id: ObjectId,
    kind: u32,
    tagger: ObjectId,
) {
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::FirstAttack {
        kind,
        unk2: 0,
        combat_tagger: tagger,
        unk3: 0,
    });
    network.send_in_range_inclusive_instance(
        actor_id,
        instance,
        FromServer::PacketSegment(ipc, actor_id),
        DestinationNetwork::ZoneClients,
    );
}

/// Whether `id` is still there to hold the claim (in the instance and alive.)
fn still_there(instance: &Instance, id: ObjectId) -> bool {
    instance
        .find_actor(id)
        .is_some_and(|actor| actor.get_common_spawn().health_points > 0)
}

/// `target` took a hostile action from `attacker`: the first one to do so (or the first since the last one left)
/// claims it. Call this before sending the action's results.
pub fn tag(
    network: &mut NetworkState,
    instance: &mut Instance,
    target: ObjectId,
    attacker: ObjectId,
) {
    if target == attacker || !still_there(instance, attacker) {
        return;
    }
    let current = match instance.find_actor(target) {
        Some(actor @ (NetworkedActor::Player { .. } | NetworkedActor::Npc { .. })) => {
            actor.get_common_spawn().combat_tagger_id.object_id
        }
        _ => return,
    };
    if current.is_valid() && still_there(instance, current) {
        return;
    }
    if let Some(actor) = instance.find_actor_mut(target) {
        actor.get_common_spawn_mut().combat_tagger_id = ObjectTypeId {
            object_id: attacker,
            ..Default::default()
        };
    }
    send(network, instance, target, TAGGED, attacker);
}

/// The fight is over for `actor_id`: nobody claims it any more.
pub fn clear(network: &mut NetworkState, instance: &mut Instance, actor_id: ObjectId) {
    let Some(actor) = instance.find_actor_mut(actor_id) else {
        return;
    };
    if !matches!(
        actor,
        NetworkedActor::Player { .. } | NetworkedActor::Npc { .. }
    ) {
        return;
    }
    let spawn = actor.get_common_spawn_mut();
    if !spawn.combat_tagger_id.object_id.is_valid() {
        return;
    }
    spawn.combat_tagger_id = ObjectTypeId::default();
    send(network, instance, actor_id, CLEARED, ObjectId(0));
}
