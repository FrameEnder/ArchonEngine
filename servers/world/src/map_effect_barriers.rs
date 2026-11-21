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
}
