//! Which way players face as they come into a duty (`resources/data/entrance_facing.json`): retail's, recorded.
//!
//! Kawari turned players the way the InstanceContent's entrance rect (or, without one, the entrance circle) faces, and
//! that was wrong in most duties: the Thousand Maws of Toto-Rak 0 where retail turned them to 1.5708, the Copperbell
//! Mines 1.5708 for 1.7881, Sastasha -1.4752 for -1.6239 (Bob's bundles of 2026-10-25 to 2026-10-30). Retail's facing
//! was the same every time a duty was entered in Bob's Chronofoil captures, so it's kept as a table, by territory.
//!
//! And which object is its entrance circle (`resources/data/entrance_circles.json`), by layout ID: an EObj ID isn't
//! one duty's circle alone (2007457 is the Porta Decumana's circle and Castrum Meridianum's way to its last hall.)

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

#[derive(Deserialize)]
struct Circles {
    territories: HashMap<u16, u32>,
}

fn circles() -> &'static Circles {
    static CIRCLES: OnceLock<Circles> = OnceLock::new();
    CIRCLES.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../resources/data/entrance_circles.json"
        ))
        .expect("resources/data/entrance_circles.json doesn't read")
    })
}

/// The layout ID of `territory`'s entrance circle, if retail's was recorded.
pub fn entrance_circle(territory: u16) -> Option<u32> {
    circles().territories.get(&territory).copied()
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
        // From the duty entries `questkit chronofoil --layers` reads (2026-11-19): Castrum Meridianum, the Praetorium,
        // the Porta Decumana.
        assert_eq!(entrance_facing(1043), Some(-1.5871));
        assert_eq!(entrance_facing(1044), Some(1.5851));
        assert_eq!(entrance_facing(1048), Some(-1.5709));
        assert_eq!(entrance_facing(132), None);
    }

    #[test]
    fn recorded_circles() {
        // Castrum Meridianum's, not the EObj 2007457 that's the Porta Decumana's circle (Bob's mark, 2026-11-19.)
        assert_eq!(entrance_circle(1043), Some(9088360));
        assert_eq!(entrance_circle(1048), Some(9088353));
        assert_eq!(entrance_circle(1042), Some(4239683)); // the Stone Vigil
        assert_eq!(entrance_circle(132), None);
    }
}
