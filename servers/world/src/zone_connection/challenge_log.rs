//! The Challenge Log window (unlock link 92, which Rising to the Challenge gives: ActorControlSelf 41 `[92, 1]` with
//! its reward, Bob's Chronofoil capture 7a4214b7 of 2026-11-17.)
//!
//! As retail answered the window in that capture (4473.7 s on, a character who had just unlocked it):
//! * it opens with client trigger 310 `[1, 0, 0, 0, 0]` (the first category) and 1301; the answers are the
//!   ChallengeLogCategory packet (opcode 395, 80 bytes: the category, then zeros) and ActorControlSelf 336 with the
//!   server's time and the week's end (1794929751 and 1795507210: the next Tuesday, 08:00 UTC);
//! * each tab pressed sends 310 `[category, 0, 0, 0, 1]`, answered with the category's packet (2, 3, 4, 5 and 8), but
//!   category 12 with ActorControlSelf 525 `[12]` and no packet.
//!
//! The entries' progress (what the 76 bytes after the category hold) isn't known: no capture has a challenge under
//! way, so every category is sent as retail sent a new character's, and nothing is counted or rewarded.

use kawari::ipc::zone::{ActorControlCategory, ServerZoneIpcData, ServerZoneIpcSegment};

use crate::ZoneConnection;

/// The categories retail answered with ActorControlSelf 525 instead of their packet (capture 7a4214b7: 12, alone of
/// the seven asked for.)
const CATEGORIES_WITHOUT_PACKET: [u32; 1] = [12];

/// The week ends on Tuesday at 08:00 UTC (retail's 1795507210 is 2026-11-24 08:00:10 UTC, sent on Tuesday the 17th
/// at 15:35 UTC.) The Unix epoch was a Thursday, so the first such Tuesday was five days and eight hours in.
const WEEK: u64 = 7 * 24 * 60 * 60;
const FIRST_WEEK_END: u64 = 5 * 24 * 60 * 60 + 8 * 60 * 60;

/// When the week `now` is in ends (Unix seconds.)
pub fn week_end(now: u64) -> u64 {
    if now < FIRST_WEEK_END {
        return FIRST_WEEK_END;
    }
    FIRST_WEEK_END + ((now - FIRST_WEEK_END) / WEEK + 1) * WEEK
}

/// Whether retail answers this category with its packet (`CATEGORIES_WITHOUT_PACKET`.)
pub fn category_has_packet(category: u32) -> bool {
    !CATEGORIES_WITHOUT_PACKET.contains(&category)
}

impl ZoneConnection {
    /// The answer to the Challenge Log asking for a category (client trigger 310.)
    pub async fn send_challenge_log_category(&mut self, category: u32) {
        if !category_has_packet(category) {
            self.actor_control_self(ActorControlCategory::ChallengeLogCategoryEmpty { category })
                .await;
            return;
        }
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ChallengeLogCategory { category });
        self.send_ipc_self(ipc).await;
    }

    /// The answer to the Challenge Log asking for its clock (client trigger 1301.)
    pub async fn send_challenge_log_times(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or(0);
        self.actor_control_self(ActorControlCategory::ChallengeLogTimes {
            now: now as u32,
            unk1: 0,
            week_ends: week_end(now) as u32,
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Retail's clock as Bob opened the Challenge Log (Chronofoil capture 7a4214b7: ActorControlSelf 336
    /// `[1794929751, 0, 1795507210]`.) Its week's end is ten seconds past the hour; the hour is what's kept.
    #[test]
    fn the_week_ends_when_retails_did() {
        assert_eq!(week_end(1794929751), 1795507200);
        assert_eq!(week_end(1795507199), 1795507200);
        assert_eq!(week_end(1795507200), 1795507200 + WEEK);
    }

    /// The categories Bob opened: 1 to 5 and 8 came as packets, 12 as ActorControlSelf 525.
    #[test]
    fn category_twelve_has_no_packet() {
        for category in [1, 2, 3, 4, 5, 8] {
            assert!(category_has_packet(category));
        }
        assert!(!category_has_packet(12));
    }
}
