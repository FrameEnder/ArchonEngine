//! Floors that give a status while a player stands on them (`resources/data/status_floors.json`): the Thousand Maws of
//! Toto-Rak's slime, at its last room's entrance and on the boss's platform, gives status 67 (param 30) on retail.
//! Kawari gave nothing (Bob's bundle 20261030-171016).
//!
//! As retail does it (Bob's Chronofoil captures caa5e908, 1fac2416 and 2528a32d, 17 times): as a player steps onto the
//! slime, ActorControl 20 (GainEffect) `[67, 30, 0]` and a StatusEffectList with it (source 0xE0000000, no time limit);
//! as they step off, ActorControl 21 (LoseEffect) `[67, 30, 0xE0000000]` and the list again; within a quarter second
//! of the position update that took them there. A jump over it gave nothing until they landed. The slime is a surface
//! of its own, raised above the floor (retail's players stood at y -38.98 to -39.13 on it, -39.29 on the platform
//! beside it), so it's told apart by its collision material (`Collision::floor_material`), found from points recorded
//! on it, rather than by a shape drawn around the recorded positions.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
};

use parking_lot::Mutex;
use serde::Deserialize;

use kawari::{
    common::{ObjectId, Position},
    ipc::zone::ActorControlCategory,
};

use crate::{
    collision::Collision,
    server::{effect::process_effects_list, instance::Instance, network::NetworkState},
};

/// How far above and below a player's feet their floor is looked for (yalms): further, they're in the air (a jump),
/// and nothing changes.
const FEET_ABOVE: f32 = 0.5;
const FEET_BELOW: f32 = 0.5;

#[derive(Deserialize)]
struct Table {
    territories: HashMap<u16, Vec<Entry>>,
}

#[derive(Deserialize)]
struct Entry {
    status: u16,
    param: u16,
    /// Points on the floor (x, y, z), recorded on retail.
    inside: Vec<[f32; 3]>,
    /// Points next to it, which mustn't be on it.
    outside: Vec<[f32; 3]>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../resources/data/status_floors.json"
        ))
        .expect("resources/data/status_floors.json doesn't read")
    })
}

/// Whether `territory` has floors that give a status (its collision needs its materials.)
pub fn has_floors(territory: u16) -> bool {
    table().territories.contains_key(&territory)
}

/// One kind of floor, as found in a zone's collision: the materials that give `status`.
#[derive(Debug, Clone)]
pub struct StatusFloor {
    status: u16,
    param: u16,
    materials: HashSet<u64>,
}

fn material_at(collision: &Collision, [x, y, z]: [f32; 3]) -> Option<u64> {
    collision
        .floor_material(x, z, y + FEET_ABOVE, y - FEET_BELOW)
        .map(|(_, material)| material)
}

/// Finds each floor's materials in `territory`'s collision from its recorded points. A floor whose outside points
/// have one of them is left out (the material isn't the floor's alone), with a warning.
fn resolve(territory: u16, collision: &Collision) -> Vec<StatusFloor> {
    let Some(entries) = table().territories.get(&territory) else {
        return Vec::new();
    };
    let mut floors = Vec::new();
    for entry in entries {
        let mut materials = HashSet::new();
        for point in &entry.inside {
            match material_at(collision, *point) {
                Some(material) => {
                    materials.insert(material);
                }
                None => tracing::warn!(
                    "Status floor {} in {territory}: no floor at {point:?}",
                    entry.status
                ),
            }
        }
        let clash: Vec<([f32; 3], u64)> = entry
            .outside
            .iter()
            .filter_map(|point| Some((*point, material_at(collision, *point)?)))
            .filter(|(_, material)| materials.contains(material))
            .collect();
        let found: Vec<String> = materials.iter().map(|m| format!("{m:#x}")).collect();
        if materials.is_empty() || !clash.is_empty() {
            tracing::warn!(
                "Status floor {} in {territory} left out: materials {found:?}, also beside it at {clash:?}",
                entry.status
            );
            continue;
        }
        tracing::info!(
            "Status floor {} (param {}) in {territory}: materials {found:?}",
            entry.status,
            entry.param
        );
        floors.push(StatusFloor {
            status: entry.status,
            param: entry.param,
            materials,
        });
    }
    floors
}

/// A player moved: on a floor that gives a status, they get it; off it, it goes.
pub fn player_moved(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    actor_id: ObjectId,
    position: Position,
) {
    let territory = instance.zone.id;
    if !has_floors(territory) {
        return;
    }
    let Some(collision) = instance.collision.clone() else {
        return;
    };
    if !collision.has_materials() {
        return;
    }
    let floors = instance
        .status_floors
        .get_or_insert_with(|| resolve(territory, &collision))
        .clone();
    if floors.is_empty() {
        return;
    }
    let p = position.0;
    let Some(material) = material_at(&collision, [p.x, p.y, p.z]) else {
        return; // in the air
    };
    let now = floors
        .iter()
        .find(|floor| floor.materials.contains(&material))
        .map(|floor| (floor.status, floor.param));
    // What the floor gave them before, if they still have it (a death takes it.)
    let before = instance
        .on_status_floor
        .get(&actor_id)
        .copied()
        .filter(|(status, _)| {
            instance
                .find_actor(actor_id)
                .and_then(|actor| actor.status_effects())
                .is_some_and(|effects| effects.get(*status).is_some())
        });
    if now == before {
        return;
    }
    if let Some((status, param)) = before {
        lose(network, instance, actor_id, status, param);
        instance.on_status_floor.remove(&actor_id);
    }
    if let Some((status, param)) = now {
        gain(network, instance, actor_id, status, param);
        instance.on_status_floor.insert(actor_id, (status, param));
    }
}

fn gain(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    actor_id: ObjectId,
    status: u16,
    param: u16,
) {
    let Some(effects) = instance
        .find_actor_mut(actor_id)
        .and_then(|actor| actor.status_effects_mut())
    else {
        return;
    };
    effects.add(status, param, 0.0, ObjectId::default());
    network.lock().send_ac_in_range_inclusive_instance(
        instance,
        actor_id,
        ActorControlCategory::GainEffect {
            effect_id: status as u32,
            param: param as u32,
            source_actor_id: ObjectId(0),
        },
    );
    process_effects_list(network.clone(), instance, actor_id);
}

fn lose(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    actor_id: ObjectId,
    status: u16,
    param: u16,
) {
    let Some(effects) = instance
        .find_actor_mut(actor_id)
        .and_then(|actor| actor.status_effects_mut())
    else {
        return;
    };
    effects.remove(status);
    network.lock().send_ac_in_range_inclusive_instance(
        instance,
        actor_id,
        ActorControlCategory::LoseEffect {
            effect_id: status as u32,
            unk2: param as u32,
            source_actor_id: ObjectId::default(),
        },
    );
    process_effects_list(network.clone(), instance, actor_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toto_raks_slime_is_recorded() {
        assert!(has_floors(1039));
        assert!(!has_floors(132));
        let slime = &table().territories[&1039][0];
        assert_eq!((slime.status, slime.param), (67, 30));
        assert!(!slime.inside.is_empty() && !slime.outside.is_empty());
    }
}
