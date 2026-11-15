//! The seasonal and special events the server's owner turns on or schedules.
//!
//! An event is a festival to the client: ZoneInit's festival IDs (and their phases) decide which of a zone's layers
//! are out (an event's NPCs and decorations), and the UI's festivals what the menus offer. Retail ran Keybound
//! Brawler as festival 257 (Connor's Chronofoil captures of 2026-11-03 to 11-09: every ZoneInit had the game's
//! festivals `[257, 256, 120]` with phases `[0, 2, 0]`.)
//!
//! * `resources/events/fork_events.json` lists the events this fork has built (their festival, phase, quests and
//!   duties.)
//! * `events.json` in the server's data folder (`$KAWARI_DATA`, beside `drop-rates.json`) is the owner's:
//!   `{"format": 1, "events": {"keybound_brawler": {"mode": "on"}}}`, a mode of `off` (the default), `on`, or
//!   `scheduled` with `start` and `end` (Unix seconds, UTC; either may be left out: from now, or for good.) It's read
//!   again whenever it changes, and the schedule is looked at as the world turns, so an event comes and goes without
//!   a restart: everyone online is told the new festivals (`ActorControlSelf` SetFestival), and a zone's layers
//!   follow at the next zone change.
//!
//! ArchonEngine ControlPanel's Dashboard -> Events writes the file.

use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

use serde_json::Value;

/// The owner's file.
pub const EVENT_FILE: &str = "events.json";

/// An event this fork has built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEvent {
    pub key: String,
    pub festival: u16,
    pub phase: u16,
    pub quests: Vec<u32>,
    pub duties: Vec<u16>,
    /// The ContentEntry events (their rows) that put one of its duties (a ContentFinderCondition) in the Duty Finder.
    pub content_entries: Vec<(u32, u16)>,
}

/// The events built (`resources/events/fork_events.json`.)
pub fn built() -> &'static [GameEvent] {
    static BUILT: OnceLock<Vec<GameEvent>> = OnceLock::new();
    BUILT.get_or_init(|| {
        let value: Value = serde_json::from_str(include_str!(
            "../../../../resources/events/fork_events.json"
        ))
        .expect("resources/events/fork_events.json doesn't read");
        let numbers = |value: &Value| -> Vec<u64> {
            value
                .as_array()
                .map(|list| list.iter().filter_map(Value::as_u64).collect())
                .unwrap_or_default()
        };
        value["events"]
            .as_array()
            .map(|events| {
                events
                    .iter()
                    .filter_map(|event| {
                        Some(GameEvent {
                            key: event["key"].as_str()?.to_string(),
                            festival: event["festival"].as_u64()? as u16,
                            phase: event["phase"].as_u64().unwrap_or(0) as u16,
                            quests: numbers(&event["quests"])
                                .into_iter()
                                .map(|id| id as u32)
                                .collect(),
                            duties: numbers(&event["duties"])
                                .into_iter()
                                .map(|id| id as u16)
                                .collect(),
                            content_entries: event["content_entries"]
                                .as_object()
                                .map(|entries| {
                                    entries
                                        .iter()
                                        .filter_map(|(row, duty)| {
                                            Some((row.parse().ok()?, duty.as_u64()? as u16))
                                        })
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// Where the owner's file is read from.
pub fn event_path() -> PathBuf {
    match std::env::var_os("KAWARI_DATA") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join(EVENT_FILE),
        _ => PathBuf::from(EVENT_FILE),
    }
}

/// The file as read, and its modification time then.
static LOADED: Mutex<Option<(Option<SystemTime>, Value)>> = Mutex::new(None);

fn loaded() -> Value {
    let path = event_path();
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
                tracing::info!("Events read from {}", path.display());
                value
            }
            Err(err) => {
                tracing::warn!(
                    "Events: {} isn't JSON ({err}); no event is on",
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

/// Is this event on at `now` (Unix seconds), by the owner's file?
pub fn is_on_in(file: &Value, key: &str, now: i64) -> bool {
    let setting = &file["events"][key];
    match setting["mode"].as_str() {
        Some("on") => true,
        Some("scheduled") => {
            let start = setting["start"].as_i64();
            let end = setting["end"].as_i64();
            start.is_none_or(|start| now >= start) && end.is_none_or(|end| now < end)
        }
        _ => false,
    }
}

/// The festivals and their phases for ZoneInit: the configured ones (`world.active_festivals`, phase 0), then each
/// event that's on, as far as the eight places go. An event whose festival is configured already keeps its place and
/// takes the event's phase.
pub fn festivals_in(
    file: &Value,
    events: &[GameEvent],
    configured: [u16; 8],
    now: i64,
) -> ([u16; 8], [u16; 8]) {
    let mut ids = [0u16; 8];
    let mut phases = [0u16; 8];
    let mut count = 0;
    for id in configured {
        if id != 0 && !ids[..count].contains(&id) && count < 8 {
            ids[count] = id;
            count += 1;
        }
    }
    for event in events {
        if !is_on_in(file, &event.key, now) {
            continue;
        }
        if let Some(place) = ids[..count].iter().position(|id| *id == event.festival) {
            phases[place] = event.phase;
        } else if count < 8 {
            ids[count] = event.festival;
            phases[count] = event.phase;
            count += 1;
        }
    }
    (ids, phases)
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|since| since.as_secs() as i64)
        .unwrap_or(0)
}

/// The festivals and phases that are on now.
pub fn festivals(configured: [u16; 8]) -> ([u16; 8], [u16; 8]) {
    festivals_in(&loaded(), built(), configured, now())
}

/// Is the event with this key on now?
pub fn is_on(key: &str) -> bool {
    is_on_in(&loaded(), key, now())
}

/// The duty a ContentEntry event (its row) enters, when it's an event's: Keybound Brawler's registrar's 120 put
/// ContentFinderCondition 1095 in the Duty Finder (Connor's Chronofoil captures, 275 times.)
pub fn content_entry_duty(row: u32) -> Option<u16> {
    built()
        .iter()
        .flat_map(|event| event.content_entries.iter())
        .find(|(entry, _)| *entry == row)
        .map(|(_, duty)| *duty)
}

/// A festival as SetFestival carries it: its ID, and its phase in the upper half (retail's `[257, 131328, 120]` in
/// Keybound Brawler's duty: 131328 is festival 256 at phase 2.)
pub fn packed(id: u16, phase: u16) -> u32 {
    id as u32 | (phase as u32) << 16
}

/// What was on when the world last looked, to tell a change (`changed`.)
static LAST: Mutex<Option<([u16; 8], [u16; 8])>> = Mutex::new(None);

/// The festivals now, if they aren't what they were at the last call (never at the first): the world tells everyone.
pub fn changed(configured: [u16; 8]) -> Option<([u16; 8], [u16; 8])> {
    let current = festivals(configured);
    let mut last = LAST.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let was = last.replace(current);
    (was.is_some() && was != Some(current)).then_some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn keybound() -> Vec<GameEvent> {
        vec![GameEvent {
            key: "keybound_brawler".to_string(),
            festival: 257,
            phase: 0,
            quests: vec![5466],
            duties: vec![1095],
            content_entries: vec![(120, 1095)],
        }]
    }

    #[test]
    fn the_built_events_read() {
        let events = built();
        let event = events
            .iter()
            .find(|event| event.key == "keybound_brawler")
            .expect("Keybound Brawler is built");
        assert_eq!(event.festival, 257);
        assert_eq!(event.quests, vec![5466]);
        assert_eq!(event.duties, vec![1095]);
        // The registrar's ContentEntry (0x2C0078) is the duty's way in.
        assert_eq!(content_entry_duty(120), Some(1095));
        assert_eq!(content_entry_duty(117), None);
    }

    #[test]
    fn off_on_and_scheduled() {
        let off = json!({});
        assert!(!is_on_in(&off, "keybound_brawler", 100));
        let on = json!({"events": {"keybound_brawler": {"mode": "on"}}});
        assert!(is_on_in(&on, "keybound_brawler", 100));
        let scheduled = json!({"events": {"keybound_brawler": {"mode": "scheduled", "start": 100, "end": 200}}});
        assert!(!is_on_in(&scheduled, "keybound_brawler", 99));
        assert!(is_on_in(&scheduled, "keybound_brawler", 100));
        assert!(is_on_in(&scheduled, "keybound_brawler", 199));
        assert!(!is_on_in(&scheduled, "keybound_brawler", 200));
        // Without a start it's on until its end; without an end, from its start for good.
        let until = json!({"events": {"keybound_brawler": {"mode": "scheduled", "end": 200}}});
        assert!(is_on_in(&until, "keybound_brawler", 0));
        let from = json!({"events": {"keybound_brawler": {"mode": "scheduled", "start": 100}}});
        assert!(is_on_in(&from, "keybound_brawler", 1 << 40));
        // A mode nobody knows is off.
        let odd = json!({"events": {"keybound_brawler": {"mode": "maybe"}}});
        assert!(!is_on_in(&odd, "keybound_brawler", 100));
    }

    #[test]
    fn an_event_s_festival_joins_the_configured_ones() {
        let on = json!({"events": {"keybound_brawler": {"mode": "on"}}});
        let configured = [120, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(
            festivals_in(&on, &keybound(), configured, 0),
            ([120, 257, 0, 0, 0, 0, 0, 0], [0; 8])
        );
        assert_eq!(
            festivals_in(&json!({}), &keybound(), configured, 0),
            (configured, [0; 8])
        );
        // Configured already: once, in its place.
        assert_eq!(
            festivals_in(&on, &keybound(), [257, 120, 0, 0, 0, 0, 0, 0], 0).0,
            [257, 120, 0, 0, 0, 0, 0, 0]
        );
        // No place left: the configured ones stay.
        assert_eq!(
            festivals_in(&on, &keybound(), [1, 2, 3, 4, 5, 6, 7, 8], 0).0,
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn a_phase_rides_in_the_upper_half() {
        // Retail's SetFestival in Keybound Brawler's duty: `[257, 131328, 120]`.
        assert_eq!(packed(256, 2), 131328);
        assert_eq!(packed(257, 0), 257);
    }
}
