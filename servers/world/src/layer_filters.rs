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
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("../../../resources/data/layer_filters.json"))
            .expect("resources/data/layer_filters.json doesn't read")
    })
}

/// The filter key for `territory` (0 if it has none), given whether each quest has been accepted or completed.
pub fn filter_key(territory: u16, reached: impl Fn(u32) -> bool) -> u32 {
    let Some(entries) = table().territories.get(&territory) else {
        return 0;
    };
    entries
        .iter()
        .filter(|entry| entry.quest.is_none_or(&reached))
        .last()
        .map(|entry| entry.key)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waking_sands_by_the_story() {
        assert_eq!(filter_key(212, |_| false), 90338);
        assert_eq!(filter_key(212, |quest| quest == 509), 90338);
        assert_eq!(filter_key(212, |quest| quest == 510 || quest == 509), 90339);
        assert_eq!(filter_key(140, |_| true), 0);
    }
}
