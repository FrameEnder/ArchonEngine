//! Party bots kept as friends: a player who sends one of their bots a friend request keeps it, so the same bot (its
//! name, class, look and glamour) can be called again from the friend list at any time (Invite to Party there: it
//! appears beside them and joins). The list is `party-bot-friends.json` in the server's data folder (`$KAWARI_DATA`,
//! beside the databases; else the working directory), by player content ID. A bot isn't a character, so it has no
//! row in the database's friend table; the friend list adds these (`docs/src/fork/party-bots.md`, "Friends"). A world
//! bot befriended is kept the same way, with its own made-up content ID (`world_bots::CONTENT_ID_BASE`.)

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};

/// The file, in the server's data folder.
pub const FRIENDS_FILE: &str = "party-bot-friends.json";

/// The top 16 bits of every bot's made-up account and content ID (`party_bot`'s `BOT_ID_BASE`.)
const BOT_ID_MASK: u64 = 0xFFFF_0000_0000_0000;
const BOT_ID_TOP: u64 = 0x00B0_0000_0000_0000;

/// Whether a content ID is a bot's made-up one: no character has one like it.
pub fn is_bot_content_id(content_id: u64) -> bool {
    let top = content_id & BOT_ID_MASK;
    top == BOT_ID_TOP || top == crate::server::world_bots::CONTENT_ID_BASE
}

/// A bot kept as a friend: what it takes to call the same one again.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedBot {
    /// Its made-up content ID, the same each time it's called (the friend list's, the party list's.)
    pub content_id: u64,
    pub name: String,
    pub classjob: u8,
    /// Its look: the customize data's 26 bytes, as hex.
    pub look: String,
    /// The glamour it wore, by its name in the glamour list (worn again if it's still there.)
    #[serde(default)]
    pub glamour: Option<String>,
    /// When it became a friend (Unix seconds, the friend list's timestamp.)
    #[serde(default)]
    pub added: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Friends {
    /// By the player's content ID (as text, for JSON.)
    #[serde(default)]
    players: HashMap<String, Vec<SavedBot>>,
}

/// Reads and writes of the file, one at a time.
static FILE_LOCK: Mutex<()> = Mutex::new(());

/// Where the file is (see the module's description.)
pub fn friends_path() -> PathBuf {
    match std::env::var_os("KAWARI_DATA") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join(FRIENDS_FILE),
        _ => PathBuf::from(FRIENDS_FILE),
    }
}

fn read(path: &Path) -> Friends {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write(path: &Path, friends: &Friends) {
    let Ok(text) = serde_json::to_string_pretty(friends) else {
        return;
    };
    // Written beside it and renamed over it, so a crash never leaves half a file.
    let temporary = path.with_extension("json.tmp");
    if std::fs::write(&temporary, text).is_ok() && std::fs::rename(&temporary, path).is_ok() {
        return;
    }
    tracing::warn!("Couldn't save the party bot friends to {}", path.display());
}

/// The bots `player` keeps as friends.
pub fn list(player: u64) -> Vec<SavedBot> {
    list_in(&friends_path(), player)
}

/// `player`'s kept bot with this content ID or, when it's 0, this name.
pub fn find(player: u64, content_id: u64, name: &str) -> Option<SavedBot> {
    find_in(&friends_path(), player, content_id, name)
}

/// Keeps `bot` as `player`'s friend. False if it already was.
pub fn add(player: u64, bot: SavedBot) -> bool {
    add_in(&friends_path(), player, bot)
}

/// Stops keeping the bot with this content ID. False if it wasn't kept.
pub fn remove(player: u64, content_id: u64) -> bool {
    remove_in(&friends_path(), player, content_id)
}

fn list_in(path: &Path, player: u64) -> Vec<SavedBot> {
    let _lock = FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    read(path)
        .players
        .remove(&player.to_string())
        .unwrap_or_default()
}

fn find_in(path: &Path, player: u64, content_id: u64, name: &str) -> Option<SavedBot> {
    list_in(path, player).into_iter().find(|bot| {
        if content_id != 0 {
            bot.content_id == content_id
        } else {
            !name.is_empty() && bot.name == name
        }
    })
}

fn add_in(path: &Path, player: u64, bot: SavedBot) -> bool {
    let _lock = FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut friends = read(path);
    let kept = friends.players.entry(player.to_string()).or_default();
    if kept.iter().any(|other| other.content_id == bot.content_id) {
        return false;
    }
    kept.push(bot);
    write(path, &friends);
    true
}

fn remove_in(path: &Path, player: u64, content_id: u64) -> bool {
    let _lock = FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut friends = read(path);
    let Some(kept) = friends.players.get_mut(&player.to_string()) else {
        return false;
    };
    let before = kept.len();
    kept.retain(|bot| bot.content_id != content_id);
    if kept.len() == before {
        return false;
    }
    write(path, &friends);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_finds_and_forgets_bots() {
        let path =
            std::env::temp_dir().join(format!("kawari-bot-friends-{}.json", fastrand::u64(..)));
        let bot = SavedBot {
            content_id: 0x00B0_0000_1234_5678,
            name: "Nylenn Jaor".to_string(),
            classjob: 24,
            look: "01".repeat(26),
            glamour: Some("Future Skypirate".to_string()),
            added: 1,
        };
        assert!(add_in(&path, 7, bot.clone()));
        assert!(!add_in(&path, 7, bot.clone()));
        assert_eq!(list_in(&path, 7), vec![bot.clone()]);
        assert!(list_in(&path, 8).is_empty());
        // By content ID, or by name when the client sends none (its party invite from a menu.)
        assert_eq!(find_in(&path, 7, bot.content_id, ""), Some(bot.clone()));
        assert_eq!(find_in(&path, 7, 0, "Nylenn Jaor"), Some(bot.clone()));
        assert_eq!(find_in(&path, 7, 0, ""), None);
        assert!(remove_in(&path, 7, bot.content_id));
        assert!(!remove_in(&path, 7, bot.content_id));
        assert!(list_in(&path, 7).is_empty());
        let _ = std::fs::remove_file(&path);

        assert!(is_bot_content_id(0x00B0_0000_1234_5678));
        assert!(!is_bot_content_id(0x0040_0000_1234_5678));
    }
}
