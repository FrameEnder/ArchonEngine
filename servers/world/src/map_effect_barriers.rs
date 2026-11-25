//! Barriers a duty's director takes down with a map effect (`resources/data/map_effect_barriers.json`).
//!
//! A map effect (DirectorMapEffect: index, timeline, state) plays a SharedGroup of the duty's layout: the one at that
//! index of the InstanceContent's ContentDirectorManagedSG. Some are barriers with collision (Castrum Meridianum's way
//! into its first boss's room, index 1). No event object owns them, so before 2026-11-21 `kawari-navimesh` built them
//! into the navmesh and the collision for good: after the way opened Player NPCs stopped at it and enemies walked on
//! through it after their patience (Bob's marks of 2026-11-20 and 2026-11-21). Those listed here are gimmicks by their
//! SharedGroup's layout ID, as an event object's are: left out of the navmesh, sealed in the collision as the duty
//! begins, and gone from it when the director sends the map effect that takes them down.

use std::collections::HashMap;
use std::sync::OnceLock;

/// For `territory`, each barrier's map effect index and the (timeline, state) that takes it down.
pub fn barriers(territory: u16) -> Vec<(u32, (u32, u32))> {
    static TABLE: OnceLock<HashMap<u16, Vec<(u32, (u32, u32))>>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let value: serde_json::Value = serde_json::from_str(include_str!(
            "../../../resources/data/map_effect_barriers.json"
        ))
        .unwrap_or_default();
        let mut table = HashMap::new();
        for (territory, indices) in value
            .get("territories")
            .and_then(|t| t.as_object())
            .into_iter()
            .flatten()
        {
            let Ok(territory) = territory.parse::<u16>() else {
                continue;
            };
            let mut list: Vec<(u32, (u32, u32))> = indices
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(index, opens)| {
                    let opens = opens.as_array()?;
                    Some((
                        index.parse().ok()?,
                        (
                            opens.first()?.as_u64()? as u32,
                            opens.get(1)?.as_u64()? as u32,
                        ),
                    ))
                })
                .collect();
            list.sort_unstable();
            table.insert(territory, list);
        }
        table
    });
    table.get(&territory).cloned().unwrap_or_default()
}

/// For `territory`, its barriers' map effect indices, sorted: what its navmesh and collision are built for
/// (`Collision::barriers_differ`.)
pub fn indices(territory: u16) -> Vec<u32> {
    barriers(territory)
        .iter()
        .map(|(index, _)| *index)
        .collect()
}

/// Whether the map effect (`index`, `timeline`, `state`) takes down one of `territory`'s barriers.
pub fn takes_down(territory: u16, index: u32, timeline: u32, state: u32) -> bool {
    barriers(territory)
        .iter()
        .any(|(barrier, opens)| *barrier == index && *opens == (timeline, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn castrum_meridianums_ways_are_barriers() {
        let castrum = barriers(1043);
        assert!(castrum.contains(&(1, (2, 1))));
        assert!(takes_down(1043, 1, 2, 1));
        // Its scenery's map effects (2 and 3: timeline 4, state 4) aren't, nor another duty's.
        assert!(!takes_down(1043, 2, 4, 4));
        assert!(!takes_down(1043, 1, 4, 4));
        assert!(barriers(1036).is_empty());
    }

    #[test]
    fn the_praetoriums_blasted_walls_are_barriers() {
        // w1d5_re.lua's map effects 2 (the five dead), 3 (the ten dead), 4 (further in) and 5 (the ride's last five),
        // timeline 2, state 1: walls blasted open (Bob's marks of 2026-11-24: nobody left by the hole.)
        assert_eq!(indices(1044), vec![2, 3, 4, 5]);
        for index in [2, 3, 4, 5] {
            assert!(takes_down(1044, index, 2, 1));
        }
        // The lift (1) and the scenery further on (0, 6 to 18) aren't.
        assert!(!takes_down(1044, 1, 2, 1));
        assert!(!takes_down(1044, 8, 2, 1));
        assert_eq!(indices(1043), vec![0, 1, 5, 6]);
    }
}

/// The states an InstanceContent's map effects begin in (DirectorSetupMapEffects) where retail's aren't all 4
/// (`resources/data/opening_map_effects.json`): the Praetorium's 6, 7, 15 and 16 begin at 8, 1, 8, 1 and Castrum
/// Meridianum's 2, 3, 8 and 9 at 1 in every run of Bob's Chronofoil captures; Kawari began every one at 4.
pub fn opening_states(instance_content: u16) -> Option<Vec<u16>> {
    static TABLE: OnceLock<HashMap<u16, Vec<u16>>> = OnceLock::new();
    TABLE
        .get_or_init(|| {
            let value: serde_json::Value = serde_json::from_str(include_str!(
                "../../../resources/data/opening_map_effects.json"
            ))
            .unwrap_or_default();
            value
                .get("instance_content")
                .and_then(|t| t.as_object())
                .into_iter()
                .flatten()
                .filter_map(|(id, states)| {
                    Some((
                        id.parse().ok()?,
                        states
                            .as_array()?
                            .iter()
                            .filter_map(|state| state.as_u64().map(|state| state as u16))
                            .collect(),
                    ))
                })
                .collect()
        })
        .get(&instance_content)
        .cloned()
}

#[cfg(test)]
mod opening_tests {
    use super::*;

    /// Retail's, as each of the Praetorium's four runs began (Bob's captures 50e72631, 6320ae56, 9ba7d981 twice.)
    #[test]
    fn the_praetoriums_map_effects_begin_as_retails() {
        let states = opening_states(86).unwrap();
        assert_eq!(states.len(), 19);
        assert_eq!((states[6], states[7], states[15], states[16]), (8, 1, 8, 1));
        assert_eq!(opening_states(15).unwrap()[2..4], [1, 1]);
        assert_eq!(opening_states(1), None);
    }
}
