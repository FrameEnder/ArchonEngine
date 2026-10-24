//! Pets (Carbuncle): dismissing them, and keeping them with their owner.

use std::time::Duration;

use glam::Vec3A;
use kawari::{
    common::{CharacterMode, ObjectId, Position, WarpType},
    ipc::zone::{
        ActorControlCategory, ActorSetPos, BattleNpcSubKind, ObjectKind, ServerZoneIpcData,
        ServerZoneIpcSegment,
    },
};

use crate::{
    ClientId, FromServer,
    server::{
        actor::{NetworkedActor, NpcState, set_character_mode},
        instance::{Instance, QueuedTaskData},
        network::{DestinationNetwork, NetworkState},
    },
};

/// The PetAction (pet hotbar) that dismisses the pet, "Away".
pub const PET_ACTION_AWAY: u32 = 1;

/// Unnamed ActorControlSelf categories retail sends the owner when the pet is dismissed, each with the pet's BNpcName:
/// 269 with 1 first, 268 with 0 last (Bob's Chronofoil capture of 2026-10-21, three dismissals.)
const PET_DISMISSED_BEGIN: u32 = 269;
const PET_DISMISSED_END: u32 = 268;

/// Retail fades a dismissed pet out 2.1 s after it falls (DeadFadeOut), and deletes it 1.8 s after that (the same
/// three dismissals.)
const PET_FADE_OUT_TIME: Duration = Duration::from_millis(2100);

/// A pet (or chocobo companion) further than this from its owner is put back beside them, as retail puts the pet back
/// beside its owner (ActorSetPos, 1.9 yalms away) after a teleport left it behind (Bob's Chronofoil capture of
/// 2026-10-21: every time it was hundreds of yalms off). Below it, the pet walks; the distance itself is a guess, as
/// retail's pet never fell this far behind otherwise.
pub const PET_CATCH_UP_DISTANCE: f32 = 50.0;

/// How far from its owner a pet put back lands (retail: 1.9 yalms.)
const PET_CATCH_UP_OFFSET: f32 = 1.9;

/// The owner's pets: battle NPCs they own that aren't party bots (Carbuncle, a chocobo companion.)
pub fn owned_by(instance: &Instance, owner: ObjectId) -> Vec<ObjectId> {
    instance
        .actors
        .iter()
        .filter(|(id, actor)| {
            matches!(actor, NetworkedActor::Npc { spawn, .. }
                if spawn.common.owner_id == owner
                    && spawn.common.mode != CharacterMode::Dead
                    && matches!(spawn.common.object_kind, ObjectKind::BattleNpc(BattleNpcSubKind::Pet)))
                && !instance.party_bots.contains_key(id)
        })
        .map(|(id, _)| *id)
        .collect()
}

/// "Away": the pet leaves, as retail does it. The owner is told its pet is gone (the pet hotbar goes away), the pet
/// falls, fades out and is deleted.
pub fn dismiss(network: &mut NetworkState, instance: &mut Instance, owner: ObjectId) {
    for pet in owned_by(instance, owner) {
        let name_id = instance
            .find_actor(pet)
            .map(|actor| actor.get_common_spawn().name_id)
            .unwrap_or_default();

        for category in [
            ActorControlCategory::Unknown {
                category: PET_DISMISSED_BEGIN,
                param1: name_id,
                param2: 1,
                param3: 0,
                param4: 0,
                param5: 0,
            },
            ActorControlCategory::SetupPet {
                owner_id: owner,
                pet_id: 0,
                pet_actor_id: ObjectId::default(),
                unk2: 1,
                unk3: 1,
            },
            ActorControlCategory::SetPetParameters {
                pet_id: 0,
                unk2: 2,
                unk3: 5,
                unk4: 7,
            },
            ActorControlCategory::Unknown {
                category: PET_DISMISSED_END,
                param1: name_id,
                param2: 0,
                param3: 0,
                param4: 0,
                param5: 0,
            },
        ] {
            network.send_to_by_actor_id(
                owner,
                FromServer::ActorControlSelf(category),
                DestinationNetwork::ZoneClients,
            );
        }

        // Retail's order: Kill, then SetMode dead.
        network.send_ac_in_range_instance(
            instance,
            pet,
            ActorControlCategory::Kill { animation_id: 0 },
        );
        set_character_mode(instance, network, pet, CharacterMode::Dead, 0, true);

        instance.insert_task(
            ClientId::default(),
            pet,
            PET_FADE_OUT_TIME,
            QueuedTaskData::DeadFadeOut {
                actor_id: pet,
                respawn_layout_id: None,
            },
        );
    }
}

/// Puts pets that have fallen too far behind their owner back beside them (see `PET_CATCH_UP_DISTANCE`.)
pub fn catch_up(network: &mut NetworkState, instance: &mut Instance) {
    let owners: Vec<(ObjectId, Vec3A, f32)> = instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, actor)| (*id, actor.position().0, actor.rotation()))
        .collect();

    for (owner, owner_position, owner_rotation) in owners {
        // A chocobo companion too.
        let mut followers = owned_by(instance, owner);
        followers.extend(instance.actors.iter().filter_map(|(id, actor)| {
            matches!(actor, NetworkedActor::Npc { spawn, .. }
                if spawn.common.owner_id == owner
                    && spawn.common.object_kind == ObjectKind::BattleNpc(BattleNpcSubKind::Chocobo))
            .then_some(*id)
        }));
        for pet in followers {
            let Some(NetworkedActor::Npc {
                spawn,
                state,
                navmesh_path,
                navmesh_path_lerp,
                last_position,
                ..
            }) = instance.find_actor_mut(pet)
            else {
                continue;
            };
            if *state != NpcState::Follow {
                continue;
            }
            let pet_position = spawn.common.position.0;
            let flat = Vec3A::new(
                pet_position.x - owner_position.x,
                0.0,
                pet_position.z - owner_position.z,
            );
            if flat.length() <= PET_CATCH_UP_DISTANCE {
                continue;
            }

            // Beside the owner, behind them.
            let position = Position(Vec3A::new(
                owner_position.x - PET_CATCH_UP_OFFSET * owner_rotation.sin(),
                owner_position.y,
                owner_position.z - PET_CATCH_UP_OFFSET * owner_rotation.cos(),
            ));
            spawn.common.position = position;
            spawn.common.rotation = owner_rotation;
            navmesh_path.clear();
            *navmesh_path_lerp = 0.0;
            *last_position = Some(position.0);

            let segment = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
                position,
                rotation: owner_rotation,
                warp_type: WarpType::None,
                ..Default::default()
            }));
            network.send_in_range_instance(
                pet,
                instance,
                FromServer::PacketSegment(segment, pet),
                DestinationNetwork::ZoneClients,
            );
        }
    }
}
