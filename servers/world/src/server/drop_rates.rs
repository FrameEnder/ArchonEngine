//! Drop rates the server's owner sets per duty: `drop-rates.json` in the server's data folder (beside
//! `glamours.txt`, `$KAWARI_DATA`; ArchonEngine ControlPanel's Dashboard → Drop rates writes it), read again whenever it
//! changes, so no restart. For now one rate: how often a coffer's mounts and minions drop (`loot::roll_contents`.)
//!
//! ```json
//! {"format": 1, "default": {"mounts_minions": 90}, "duties": {"57": {"mounts_minions": 100}}}
//! ```
//!
//! `duties` is by ContentFinderCondition ID; a duty not in it takes `default`, and without the file (or a value) it's
//! `MOUNTS_MINIONS_PERCENT`.

use std::{path::PathBuf, sync::Mutex, time::SystemTime};

use serde_json::Value;

/// The file's name, in the server's data folder.
pub const DROP_RATE_FILE: &str = "drop-rates.json";

/// How often (percent) a mount or minion in a dungeon's or trial's coffer drops, unless the file says otherwise (Bob,
/// 2026-11-10: 90 %.)
pub const MOUNTS_MINIONS_PERCENT: u8 = 90;

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

/// A rate from the file (`rates`) for a duty: its own, else the default, else `fallback`; clamped to 0-100.
pub fn rate_in(rates: &Value, content_finder_condition: u16, key: &str, fallback: u8) -> u8 {
    let percent = |value: Option<&Value>| value.and_then(Value::as_f64);
    percent(
        rates
            .get("duties")
            .and_then(|duties| duties.get(content_finder_condition.to_string()))
            .and_then(|duty| duty.get(key)),
    )
    .or_else(|| percent(rates.get("default").and_then(|default| default.get(key))))
    .map(|percent| percent.clamp(0.0, 100.0).round() as u8)
    .unwrap_or(fallback)
}

/// How often (percent) a mount or minion in this duty's coffers drops.
pub fn mounts_minions_percent(content_finder_condition: u16) -> u8 {
    rate_in(
        &loaded(),
        content_finder_condition,
        "mounts_minions",
        MOUNTS_MINIONS_PERCENT,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_duty_its_own_then_the_default_then_ninety() {
        let rates = serde_json::json!({
            "format": 1,
            "default": {"mounts_minions": 50},
            "duties": {"57": {"mounts_minions": 100}, "4": {"mounts_minions": 140}}
        });
        assert_eq!(rate_in(&rates, 57, "mounts_minions", 90), 100);
        assert_eq!(rate_in(&rates, 4, "mounts_minions", 90), 100);
        assert_eq!(rate_in(&rates, 2, "mounts_minions", 90), 50);
        assert_eq!(rate_in(&Value::Null, 2, "mounts_minions", 90), 90);
    }
}
