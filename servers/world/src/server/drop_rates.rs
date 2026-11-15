//! The rates the server's owner sets: how often and how much items drop, and how much gil and EXP is given
//! (`drop-rates.json` in the server's data folder, beside `glamours.txt`, `$KAWARI_DATA`; ArchonEngine ControlPanel's
//! Dashboard → Drop rates writes it). Read again whenever it changes, so no restart.
//!
//! Every rate but one is a **multiplier in percent**: 100 is the server's own (retail's, where it's known), 200 twice
//! as often or as much, 0 never. Without the file, or where it says nothing, everything is 100.
//!
//! ```json
//! {"format": 2,
//!  "items": {"all":        {"all": 100, "group:6": 150, "cat:58": 50, "mounts_minions": 90},
//!            "expansions": {"0": {"all": 120}},
//!            "types":      {"2": {"group:3": 200}, "world": {"all": 80}},
//!            "duties":     {"4": {"all": 300, "mounts_minions": 100}},
//!            "zones":      {"148": {"cat:45": 0}}},
//!  "gil": {"all": 100, "kills": 150},
//!  "exp": {"all": 200, "quests": 100}}
//! ```
//!
//! **Items.** A drop happens somewhere (`DropPlace`: a duty, by ContentFinderCondition, or an open-world zone, by
//! TerritoryType; its content type, or `world` out in the open; its expansion) and is of some kind (`ItemClass`: its
//! ItemUICategory, `cat:N`, and that category's group in the game's own order, ItemUICategory.OrderMajor, `group:N`).
//! **The nearest setting wins**: the duty's or zone's own, then its content type's, then its expansion's, then
//! `all`; and in the first of those that says anything for the item, its category's rate, else its group's, else that
//! place's `all`. What uses it: a coffer's items (`loot::roll_contents`) and what an enemy drops
//! (`zone_connection/quest.rs`, `scale_drops`).
//!
//! `mounts_minions` is the one that isn't a multiplier: the chance in percent that a mount or minion a coffer may
//! hold drops, 90 unless a place says otherwise (Bob, 2026-11-10), looked up through the same places.
//!
//! **Gil and EXP.** `all`, and a rate of its own for a source that overrides it: gil from `kills`, `quests`,
//! `duties` (a clear's bonus, a roulette's) and `fates`; EXP from those four and `gathering`, `crafting`,
//! `hunting_log` and `other` (what a script gives).
//!
//! The file of format 1 (`{"default": {"mounts_minions": 90}, "duties": {"57": {"mounts_minions": 100}}}`) is still
//! read: its two parts are format 2's `items.all` and `items.duties`.

use std::{path::PathBuf, sync::Mutex, time::SystemTime};

use serde_json::Value;

/// The file's name, in the server's data folder.
pub const DROP_RATE_FILE: &str = "drop-rates.json";

/// How often (percent) a mount or minion in a dungeon's or trial's coffer drops, unless the file says otherwise (Bob,
/// 2026-11-10: 90 %.)
pub const MOUNTS_MINIONS_PERCENT: u8 = 90;

/// The most a multiplier can be: ten times.
pub const MOST_PERCENT: u32 = 1000;

/// The key of the open world among `items.types` (the others are ContentType rows.)
pub const WORLD: &str = "world";

/// Where a drop happens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DropPlace {
    /// The territory's expansion (TerritoryType.ExVersion: 0 is A Realm Reborn.)
    pub expansion: u8,
    /// The duty's ContentFinderCondition and its ContentType, in a duty.
    pub duty: Option<(u16, u16)>,
    /// The TerritoryType.
    pub zone: u16,
}

/// What kind of item drops.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ItemClass {
    /// Its ItemUICategory.
    pub category: u8,
    /// That category's group (ItemUICategory.OrderMajor.)
    pub group: u8,
}

/// The sources of gil and of EXP a rate can be set for.
pub const GIL_SOURCES: [&str; 4] = ["kills", "quests", "duties", "fates"];
pub const EXP_SOURCES: [&str; 8] = [
    "kills",
    "quests",
    "duties",
    "fates",
    "gathering",
    "crafting",
    "hunting_log",
    "other",
];

/// Where the file is read from.
pub fn drop_rate_path() -> PathBuf {
    match std::env::var_os("KAWARI_DATA") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join(DROP_RATE_FILE),
        _ => PathBuf::from(DROP_RATE_FILE),
    }
}

/// The file as read, and its modification time then.
static LOADED: Mutex<Option<(Option<SystemTime>, Value)>> = Mutex::new(None);

fn loaded() -> Value {
    let path = drop_rate_path();
    let modified = std::fs::metadata(&path)
        .and_then(|meta| meta.modified())
        .ok();
    let mut loaded = LOADED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((at, value)) = loaded.as_ref()
        && *at == modified
    {
        return value.clone();
    }
    let value = match std::fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => {
                tracing::info!("Drop rates read from {}", path.display());
                value
            }
            Err(err) => {
                tracing::warn!(
                    "Drop rates: {} isn't JSON ({err}); the defaults apply",
                    path.display()
                );
                Value::Null
            }
        },
        Err(_) => Value::Null,
    };
    *loaded = Some((modified, value.clone()));
    value
}

/// The places' settings for a drop there, nearest first: the duty's or zone's own, its content type's (or the open
/// world's), its expansion's, everywhere's. Format 1's `duties` and `default` are the first and last of a duty's.
fn settings<'a>(rates: &'a Value, place: &DropPlace) -> Vec<&'a Value> {
    let items = rates.get("items");
    let under = |part: &str, key: String| items.and_then(|items| items.get(part)?.get(key));
    let mut found = Vec::new();
    match place.duty {
        Some((duty, content_type)) => {
            found.push(under("duties", duty.to_string()));
            found.push(rates.get("duties").and_then(|d| d.get(duty.to_string())));
            found.push(under("types", content_type.to_string()));
        }
        None => {
            found.push(under("zones", place.zone.to_string()));
            found.push(under("types", WORLD.to_string()));
        }
    }
    found.push(under("expansions", place.expansion.to_string()));
    found.push(items.and_then(|items| items.get("all")));
    found.push(rates.get("default"));
    found.into_iter().flatten().collect()
}

fn number(value: Option<&Value>) -> Option<f64> {
    value.and_then(Value::as_f64).filter(|n| n.is_finite())
}

/// How often and how much an item of this kind drops in this place, in percent of the server's own (see the module's
/// notes: the nearest place that says anything for it; its category, else its group, else that place's `all`.)
pub fn item_percent_in(rates: &Value, place: &DropPlace, item: &ItemClass) -> u32 {
    for setting in settings(rates, place) {
        let rate = number(setting.get(format!("cat:{}", item.category)))
            .or_else(|| number(setting.get(format!("group:{}", item.group))))
            .or_else(|| number(setting.get("all")));
        if let Some(rate) = rate {
            return rate.clamp(0.0, MOST_PERCENT as f64).round() as u32;
        }
    }
    100
}

/// The chance (percent) that a mount or minion a coffer may hold drops in this place: the nearest place's
/// `mounts_minions`, else `MOUNTS_MINIONS_PERCENT`.
pub fn mounts_minions_percent_in(rates: &Value, place: &DropPlace) -> u8 {
    settings(rates, place)
        .into_iter()
        .find_map(|setting| number(setting.get("mounts_minions")))
        .map(|percent| percent.clamp(0.0, 100.0).round() as u8)
        .unwrap_or(MOUNTS_MINIONS_PERCENT)
}

/// The rate (percent) of gil (`kind` "gil") or EXP ("exp") from a source: the source's own, else `all`, else 100.
pub fn reward_percent_in(rates: &Value, kind: &str, source: &str) -> u32 {
    let part = rates.get(kind);
    number(part.and_then(|part| part.get(source)))
        .or_else(|| number(part.and_then(|part| part.get("all"))))
        .map(|rate| rate.clamp(0.0, MOST_PERCENT as f64).round() as u32)
        .unwrap_or(100)
}

/// An amount at a rate, to the nearest whole (gil, EXP.)
pub fn scale(amount: u32, percent: u32) -> u32 {
    ((amount as u64 * percent as u64 + 50) / 100).min(u32::MAX as u64) as u32
}

/// How many of something that comes `count` at a time come at a rate: the whole part of `count × percent`, and one
/// more as often as the rest says (`roll` in 0..1), so 50 % halves how often one drops and 250 % of one is two, and a
/// third half the time.
pub fn scale_count(count: u32, percent: u32, roll: f32) -> u32 {
    let hundredths = count as u64 * percent as u64;
    let whole = hundredths / 100;
    let rest = (hundredths % 100) as f32 / 100.0;
    (whole + u64::from(roll < rest)).min(u32::MAX as u64) as u32
}

/// How often and how much an item of this kind drops in this place (percent.)
pub fn item_percent(place: &DropPlace, item: &ItemClass) -> u32 {
    item_percent_in(&loaded(), place, item)
}

/// How often (percent) a mount or minion in this place's coffers drops.
pub fn mounts_minions_percent(place: &DropPlace) -> u8 {
    mounts_minions_percent_in(&loaded(), place)
}

/// Gil from a source (`GIL_SOURCES`) at the owner's rate.
pub fn gil(amount: u32, source: &str) -> u32 {
    debug_assert!(
        GIL_SOURCES.contains(&source),
        "not a source of gil: {source}"
    );
    scale(amount, reward_percent_in(&loaded(), "gil", source))
}

/// EXP from a source (`EXP_SOURCES`) at the owner's rate.
pub fn exp(amount: u32, source: &str) -> u32 {
    debug_assert!(
        EXP_SOURCES.contains(&source),
        "not a source of EXP: {source}"
    );
    scale(amount, reward_percent_in(&loaded(), "exp", source))
}

/// The same for the places that count EXP signed.
pub fn exp_signed(amount: i32, source: &str) -> i32 {
    exp(amount.max(0) as u32, source).min(i32::MAX as u32) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    const SASTASHA: DropPlace = DropPlace {
        expansion: 0,
        duty: Some((4, 2)),
        zone: 1036,
    };
    const SHROUD: DropPlace = DropPlace {
        expansion: 0,
        duty: None,
        zone: 148,
    };
    const MATERIA: ItemClass = ItemClass {
        category: 58,
        group: 7,
    };
    const BODY: ItemClass = ItemClass {
        category: 35,
        group: 3,
    };

    #[test]
    fn the_nearest_setting_wins() {
        let rates = serde_json::json!({
            "format": 2,
            "items": {
                "all": {"all": 110, "cat:58": 50},
                "expansions": {"0": {"group:3": 120}, "1": {"all": 5}},
                "types": {"2": {"cat:35": 200}, "world": {"all": 80}},
                "duties": {"4": {"all": 300}, "9": {"mounts_minions": 100}},
                "zones": {"148": {"cat:45": 0}}
            }
        });
        // The duty's own blanket rate, though everywhere has one for materia.
        assert_eq!(item_percent_in(&rates, &SASTASHA, &MATERIA), 300);
        // Another dungeon: its type's for bodies; its expansion's for the rest of the armour; everywhere's for materia.
        let tam_tara = DropPlace {
            duty: Some((2, 2)),
            ..SASTASHA
        };
        assert_eq!(item_percent_in(&rates, &tam_tara, &BODY), 200);
        let legs = ItemClass {
            category: 36,
            group: 3,
        };
        assert_eq!(item_percent_in(&rates, &tam_tara, &legs), 120);
        assert_eq!(item_percent_in(&rates, &tam_tara, &MATERIA), 50);
        // A trial there: nothing nearer than everywhere's.
        let trial = DropPlace {
            duty: Some((56, 4)),
            ..SASTASHA
        };
        assert_eq!(
            item_percent_in(
                &rates,
                &trial,
                &ItemClass {
                    category: 61,
                    group: 7
                }
            ),
            110
        );
        // Out in the world: the zone's own for one category, the open world's for the rest.
        let ingredient = ItemClass {
            category: 45,
            group: 6,
        };
        assert_eq!(item_percent_in(&rates, &SHROUD, &ingredient), 0);
        assert_eq!(item_percent_in(&rates, &SHROUD, &MATERIA), 80);
        // Nothing said: as the server's own. Too much: ten times at most.
        assert_eq!(item_percent_in(&Value::Null, &SHROUD, &MATERIA), 100);
        let wild = serde_json::json!({"items": {"all": {"all": 99999}}});
        assert_eq!(item_percent_in(&wild, &SHROUD, &MATERIA), MOST_PERCENT);
    }

    #[test]
    fn mounts_and_minions_are_a_chance_and_format_1_is_still_read() {
        let rates = serde_json::json!({
            "format": 2,
            "items": {"all": {"mounts_minions": 50, "all": 300},
                      "types": {"4": {"mounts_minions": 75}},
                      "duties": {"57": {"mounts_minions": 100}, "4": {"mounts_minions": 140}}}
        });
        let duty = |duty, kind| DropPlace {
            expansion: 0,
            duty: Some((duty, kind)),
            zone: 0,
        };
        assert_eq!(mounts_minions_percent_in(&rates, &duty(57, 4)), 100);
        assert_eq!(mounts_minions_percent_in(&rates, &duty(4, 2)), 100);
        assert_eq!(mounts_minions_percent_in(&rates, &duty(58, 4)), 75);
        assert_eq!(mounts_minions_percent_in(&rates, &duty(2, 2)), 50);
        assert_eq!(mounts_minions_percent_in(&Value::Null, &duty(2, 2)), 90);
        // The file as Kawari Web 2.27.0 wrote it.
        let old = serde_json::json!({
            "format": 1,
            "default": {"mounts_minions": 50},
            "duties": {"57": {"mounts_minions": 100}}
        });
        assert_eq!(mounts_minions_percent_in(&old, &duty(57, 4)), 100);
        assert_eq!(mounts_minions_percent_in(&old, &duty(2, 2)), 50);
        assert_eq!(item_percent_in(&old, &duty(2, 2), &MATERIA), 100);
    }

    #[test]
    fn gil_and_exp_by_source() {
        let rates = serde_json::json!({
            "gil": {"all": 50, "kills": 300},
            "exp": {"quests": 200}
        });
        assert_eq!(reward_percent_in(&rates, "gil", "kills"), 300);
        assert_eq!(reward_percent_in(&rates, "gil", "quests"), 50);
        assert_eq!(reward_percent_in(&rates, "exp", "quests"), 200);
        assert_eq!(reward_percent_in(&rates, "exp", "kills"), 100);
        assert_eq!(reward_percent_in(&Value::Null, "exp", "kills"), 100);
        assert_eq!(scale(55, 150), 83);
        assert_eq!(scale(1, 40), 0);
        assert_eq!(scale(u32::MAX, 1000), u32::MAX);
    }

    #[test]
    fn a_count_at_a_rate() {
        // Half as often.
        assert_eq!(scale_count(1, 50, 0.49), 1);
        assert_eq!(scale_count(1, 50, 0.5), 0);
        // Two and a half times: two, and a third half the time.
        assert_eq!(scale_count(1, 250, 0.49), 3);
        assert_eq!(scale_count(1, 250, 0.9), 2);
        assert_eq!(scale_count(3, 100, 0.0), 3);
        assert_eq!(scale_count(3, 0, 0.0), 0);
    }
}
