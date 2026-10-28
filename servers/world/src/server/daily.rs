//! Daily rewards and their reset: retail's day starts at 15:00 UTC (8 AM in Arizona, which keeps no daylight saving
//! time; 3 PM GMT, the FFXIV Console Games Wiki's *Duty Roulette*), the same for everything daily. What each character
//! has claimed since is kept in `daily-rewards.json` in the server's data folder (`$KAWARI_DATA`, beside the
//! databases; else the working directory), by content ID and a key naming the reward (`roulette-1` for Duty Roulette:
//! Leveling), with when it was claimed. A reward can be claimed again once a reset has passed since, so the day turns
//! over at 15:00 UTC whether or not the server was running then (`docs/src/fork/duty-finder.md`).

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

/// The file, in the server's data folder.
pub const DAILY_FILE: &str = "daily-rewards.json";

/// When the day turns over, in seconds after midnight UTC (15:00.)
pub const RESET_SECONDS_UTC: u64 = 15 * 3600;
const DAY: u64 = 24 * 3600;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Claims {
    /// By the character's content ID (as text, for JSON): each reward's key and when it was last claimed (Unix
    /// seconds.)
    #[serde(default)]
    players: HashMap<String, HashMap<String, u64>>,
}

static FILE_LOCK: Mutex<()> = Mutex::new(());

/// Where the file is (see the module's description.)
pub fn daily_path() -> PathBuf {
    match std::env::var_os("KAWARI_DATA") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join(DAILY_FILE),
        _ => PathBuf::from(DAILY_FILE),
    }
}

/// The most recent reset at or before `now` (Unix seconds.)
pub fn last_reset(now: u64) -> u64 {
    if now < RESET_SECONDS_UTC {
        return 0;
    }
    (now - RESET_SECONDS_UTC) / DAY * DAY + RESET_SECONDS_UTC
}

/// The next reset after `now`.
pub fn next_reset(now: u64) -> u64 {
    last_reset(now) + DAY
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|time| time.as_secs())
        .unwrap_or_default()
}

fn read(path: &Path) -> Claims {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write(path: &Path, claims: &Claims) {
    let Ok(text) = serde_json::to_string_pretty(claims) else {
        return;
    };
    // Written beside it and renamed over it, so a crash never leaves half a file.
    let temporary = path.with_extension("json.tmp");
    if std::fs::write(&temporary, text).is_ok() && std::fs::rename(&temporary, path).is_ok() {
        return;
    }
    tracing::warn!("Couldn't save the daily rewards to {}", path.display());
}

/// Claims `player`'s daily `key` now: true (and kept) if it wasn't claimed since the last reset, false if it was.
pub fn claim(player: u64, key: &str) -> bool {
    claim_in(&daily_path(), player, key, now_secs())
}

/// Whether `player` may still claim `key` today.
pub fn available(player: u64, key: &str) -> bool {
    available_in(&daily_path(), player, key, now_secs())
}

fn available_in(path: &Path, player: u64, key: &str, now: u64) -> bool {
    let _lock = FILE_LOCK.lock();
    read(path)
        .players
        .get(&player.to_string())
        .and_then(|claims| claims.get(key))
        .is_none_or(|claimed| *claimed < last_reset(now))
}

fn claim_in(path: &Path, player: u64, key: &str, now: u64) -> bool {
    let _lock = FILE_LOCK.lock();
    let mut claims = read(path);
    let theirs = claims.players.entry(player.to_string()).or_default();
    if theirs
        .get(key)
        .is_some_and(|claimed| *claimed >= last_reset(now))
    {
        return false;
    }
    theirs.insert(key.to_string(), now);
    write(path, &claims);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-10-28 15:00:00 UTC.
    const RESET: u64 = 1_793_199_600;

    #[test]
    fn the_day_turns_at_15_utc() {
        assert_eq!(last_reset(RESET), RESET);
        assert_eq!(last_reset(RESET - 1), RESET - DAY);
        assert_eq!(last_reset(RESET + DAY - 1), RESET);
        assert_eq!(next_reset(RESET + 5), RESET + DAY);
        // 08:00 in Arizona (UTC-7) is 15:00 UTC.
        assert_eq!(RESET % DAY, RESET_SECONDS_UTC);
    }

    #[test]
    fn claims_once_a_day_and_survive_restarts() {
        let dir = std::env::temp_dir().join(format!("kawari-daily-{}", fastrand::u64(..)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(DAILY_FILE);
        assert!(claim_in(&path, 7, "roulette-1", RESET + 60));
        assert!(!claim_in(&path, 7, "roulette-1", RESET + 3600));
        assert!(!available_in(&path, 7, "roulette-1", RESET + DAY - 1));
        // Another reward, another character.
        assert!(claim_in(&path, 7, "roulette-6", RESET + 3600));
        assert!(claim_in(&path, 8, "roulette-1", RESET + 3600));
        // The next day (read again from the file, as after a restart.)
        assert!(available_in(&path, 7, "roulette-1", RESET + DAY));
        assert!(claim_in(&path, 7, "roulette-1", RESET + DAY));
        std::fs::remove_dir_all(&dir).ok();
    }
}
