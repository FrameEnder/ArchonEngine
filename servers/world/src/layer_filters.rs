//! Which layer filter a zone is shown with (`resources/data/layer_filters.json`).
//!
//! A zone's layout (its LVB) can list several layer filters ("layer sets": an ID, a territory and a duty), and each of
//! its layers shows only under some of them (FFXIVClientStructs: `FileLayerGroupLayerFilter`, Match or NoMatch on a
//! list of keys; `LayoutManager.LayerFilterKey`). Where the story changes a place, as the Waking Sands, retail's server
//! tells the client which filter to use by quest progress: ZoneInit's and ActorSetPos' `transition_territory_filter_key`.
//! Kawari sent 0, and the client showed every stage's people at once (Bob, 2026-10-26).

use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;

#[derive(Deserialize)]
struct Table {
    territories: HashMap<u16, Vec<Entry>>,
}

#[derive(Deserialize)]
struct Entry {
    key: u32,
    /// Applies once this quest (its short ID, as in the journal) has been accepted or completed.
    #[serde(default)]
    quest: Option<u32>,
    /// With `quest`: only once it has reached this sequence (or been completed), e.g. 255 after a cutscene.
    #[serde(default)]
    sequence: Option<u8>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("../../../resources/data/layer_filters.json"))
            .expect("resources/data/layer_filters.json doesn't read")
    })
}

/// Whether `territory` has layer filters chosen by the story (its key can change while the player is there.)
pub fn has_filters(territory: u16) -> bool {
    table().territories.contains_key(&territory)
}

/// The filter key for `territory` (0 if it has none), given whether each quest has been reached: `reached(quest,
/// sequence)` is whether it has been accepted (and, with a sequence, got that far) or completed.
pub fn filter_key(territory: u16, reached: impl Fn(u32, Option<u8>) -> bool) -> u32 {
    let Some(entries) = table().territories.get(&territory) else {
        return 0;
    };
    entries
        .iter()
        .filter(|entry| {
            entry
                .quest
                .is_none_or(|quest| reached(quest, entry.sequence))
        })
        .last()
        .map(|entry| entry.key)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waking_sands_by_the_story() {
        assert_eq!(filter_key(212, |_, _| false), 90338);
        assert_eq!(filter_key(212, |quest, _| quest == 509), 90338);
        assert_eq!(
            filter_key(212, |quest, _| quest == 510 || quest == 509),
            90339
        );
        // Bob's capture b8432613: A Hero in the Making, then the Grand Company's next step, then Sylph-management.
        assert_eq!(filter_key(212, |quest, _| quest <= 511), 90340);
        assert_eq!(
            filter_key(212, |quest, _| quest <= 511 || quest == 684),
            90341
        );
        assert_eq!(
            filter_key(212, |quest, _| quest <= 513 || quest == 683),
            90342
        );
        // Back from the Wood (746): its Waking Sands cutscene (sequence 255) moves it on; accepted isn't enough.
        let back_from_the_wood = |at: u8| {
            move |quest: u32, sequence: Option<u8>| {
                quest <= 513 || quest == 683 || (quest == 746 && sequence.is_none_or(|s| at >= s))
            }
        };
        assert_eq!(filter_key(212, back_from_the_wood(2)), 90342);
        assert_eq!(filter_key(212, back_from_the_wood(255)), 90343);
        // Wrath of the Titan (516), then 517 (92218, every visit at its sequence 2) and Bringing out the Dead (876).
        assert_eq!(
            filter_key(212, |quest, _| quest <= 516 || quest == 683),
            90344
        );
        assert_eq!(
            filter_key(212, |quest, _| quest <= 517 || quest == 683),
            92218
        );
        assert_eq!(
            filter_key(212, |quest, _| quest <= 518 || quest == 683 || quest == 876),
            92219
        );
        assert_eq!(filter_key(140, |_, _| true), 0);
    }
}
