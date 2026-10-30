//! Which way players face as they come into a duty (`resources/data/entrance_facing.json`): retail's, recorded.
//!
//! Kawari turned players the way the InstanceContent's entrance rect (or, without one, the entrance circle) faces, and
//! that was wrong in most duties: the Thousand Maws of Toto-Rak 0 where retail turned them to 1.5708, the Copperbell
//! Mines 1.5708 for 1.7881, Sastasha -1.4752 for -1.6239 (Bob's bundles of 2026-10-25 to 2026-10-30). Retail's facing
//! was the same every time a duty was entered in Bob's Chronofoil captures, so it's kept as a table, by territory.

use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;

#[derive(Deserialize)]
struct Table {
    territories: HashMap<u16, f32>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("../../../resources/data/entrance_facing.json"))
            .expect("resources/data/entrance_facing.json doesn't read")
    })
}

/// Retail's facing at `territory`'s entrance, if it was recorded.
pub fn entrance_facing(territory: u16) -> Option<f32> {
    table().territories.get(&territory).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_duties() {
        assert_eq!(entrance_facing(1039), Some(1.5708)); // the Thousand Maws of Toto-Rak
        assert_eq!(entrance_facing(1038), Some(1.7881)); // the Copperbell Mines
        assert_eq!(entrance_facing(1036), Some(-1.6239)); // Sastasha
        assert_eq!(entrance_facing(1037), Some(0.0)); // the Tam-Tara Deepcroft
        assert_eq!(entrance_facing(1045), Some(1.5708)); // the Bowl of Embers
        assert_eq!(entrance_facing(132), None);
    }
}
