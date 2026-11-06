//! What a key item does to the battle NPC it's used on (`resources/data/event_item_effects.json`): Factual Folklore's
//! ITEM0 on its giant, statuses and its health down to a fifth, as retail's ActionEffect and EffectResult showed (Bob's
//! Chronofoil captures 130088ec and 3c56e09f, 2026-11-06). Used by `server::action::use_event_item`.

use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;

#[derive(Deserialize)]
struct Table {
    items: HashMap<u32, Effects>,
}

/// One status the NPC gains.
#[derive(Deserialize, Clone, Copy)]
pub struct Status {
    pub id: u16,
    pub duration: f32,
    /// The GainEffect entry's first byte, as retail sent it.
    #[serde(default)]
    pub unk1: u8,
}

#[derive(Deserialize)]
pub struct Effects {
    #[serde(default)]
    pub statuses: Vec<Status>,
    /// Its health and maximum health become this share of its maximum, rounded down.
    #[serde(default)]
    pub health_percent: Option<u32>,
    /// The ActionEffect's animation lock.
    #[serde(default)]
    pub animation_lock: Option<f32>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../resources/data/event_item_effects.json"
        ))
        .expect("resources/data/event_item_effects.json doesn't read")
    })
}

/// What `item` does to a battle NPC it's used on, if anything.
pub fn effects(item: u32) -> Option<&'static Effects> {
    table().items.get(&item)
}

/// The health a battle NPC is left with: (health, maximum).
pub fn weakened(effects: &Effects, health: u32, max_health: u32) -> (u32, u32) {
    match effects.health_percent {
        Some(percent) => {
            let max = (max_health as u64 * percent as u64 / 100) as u32;
            (health.min(max), max)
        }
        None => (health, max_health),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factual_folklore_item() {
        let effects = effects(2000961).unwrap();
        assert_eq!(effects.statuses.len(), 2);
        assert_eq!(effects.statuses[0].id, 22);
        // Retail: 14283 HP, then 2856 and 2856.
        assert_eq!(weakened(effects, 13351, 14283), (2856, 2856));
        assert_eq!(weakened(effects, 1000, 14283), (1000, 2856));
        assert!(super::effects(1).is_none());
    }
}
