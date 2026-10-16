//! Achievements: the completed ones are saved per character (`unlock.achievements`, newest five in
//! `unlock.achievement_history`), sent when the client asks (the Achievements window), and completed when the
//! character meets what the Achievement sheet asks for.
//!
//! What each Achievement sheet Type asks for, as the community's sheet definitions (SaintCoinach) describe it. Only
//! these are checked; the rest (counters for kills, crafts and so on) can only be given with `//gm achievement`:
//! * 2: every achievement in Data
//! * 3: the class or job Data[0] at level Data[1] or above
//! * 6: every quest in Data
//! * 9: any quest in Data

use kawari::{
    constants::{ACHIEVEMENT_BITMASK_SIZE, ACHIEVEMENT_HISTORY_SIZE},
    ipc::zone::{ActorControlCategory, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{ZoneConnection, common::adjust_quest_id, gamedata::AchievementRule};

/// The highest achievement the client's bitmap has room for (it ignores higher ones.)
const MAX_ACHIEVEMENT_ID: u32 = ACHIEVEMENT_BITMASK_SIZE as u32 * 8 - 1;

const KIND_ACHIEVEMENTS: u8 = 2;
const KIND_CLASS_LEVEL: u8 = 3;
const KIND_ALL_QUESTS: u8 = 6;
const KIND_ANY_QUEST: u8 = 9;

impl ZoneConnection {
    pub fn has_achievement(&self, id: u32) -> bool {
        let achievements = &self.player_data.unlock.achievements;
        (id as usize / 8) < achievements.data.len() && achievements.contains(id)
    }

    /// The last five completed, newest first.
    fn achievement_history(&self) -> [u16; 5] {
        let mut data = self.player_data.unlock.achievement_history.data.clone();
        data.resize(ACHIEVEMENT_HISTORY_SIZE, 0);
        std::array::from_fn(|i| u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]))
    }

    /// Sends the completed achievements, the reply to the client asking for them.
    pub async fn send_achievements(&mut self) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::Achievements {
            completed: self.player_data.unlock.achievements.data_truncated(),
            history: self.achievement_history(),
            unk: [0; 26],
        });
        self.send_ipc_self(ipc).await;
    }

    /// Completes an achievement. With `announce`, the client shows the "Achievement unlocked" pop-up; without, it only
    /// learns about it the next time it asks.
    pub async fn complete_achievement(&mut self, id: u32, announce: bool) {
        if id == 0 || id > MAX_ACHIEVEMENT_ID || self.has_achievement(id) {
            return;
        }

        let achievements = &mut self.player_data.unlock.achievements;
        achievements.data.resize(ACHIEVEMENT_BITMASK_SIZE, 0);
        achievements.set(id);

        let mut history = self.achievement_history();
        history.rotate_right(1);
        history[0] = id as u16;
        self.player_data.unlock.achievement_history.data =
            history.iter().flat_map(|id| id.to_le_bytes()).collect();

        tracing::info!("Achievement {id} completed");
        if announce {
            self.actor_control_self(ActorControlCategory::AchievementCompleted { id })
                .await;
        }
    }

    /// Takes an achievement away (the client sees it the next time it asks.)
    pub fn clear_achievement(&mut self, id: u32) {
        if !self.has_achievement(id) {
            return;
        }
        self.player_data.unlock.achievements.clear(id);
        let history: Vec<u16> = self
            .achievement_history()
            .into_iter()
            .filter(|entry| *entry as u32 != id)
            .collect();
        self.player_data.unlock.achievement_history.data =
            history.iter().flat_map(|id| id.to_le_bytes()).collect();
    }

    /// How far along an achievement is, as (current, max), for the kinds that are checked; 1 of 1 for one that's
    /// complete and 0 of 0 for the rest, which the client shows without a count.
    fn achievement_progress(&self, rule: &AchievementRule) -> (u32, u32) {
        let listed = rule.data.iter().filter(|x| **x > 0).map(|x| *x as u32);
        match rule.kind {
            KIND_ACHIEVEMENTS => {
                let ids: Vec<u32> = listed.collect();
                let done = ids.iter().filter(|id| self.has_achievement(**id)).count();
                (done as u32, ids.len() as u32)
            }
            KIND_CLASS_LEVEL => {
                let level = self.level_for(rule.data[0] as u16).unwrap_or(0) as u32;
                let wanted = rule.data[1].max(0) as u32;
                (level.min(wanted), wanted)
            }
            KIND_ALL_QUESTS => {
                let ids: Vec<u32> = listed.collect();
                let done = ids.iter().filter(|id| self.quest_completed(**id)).count();
                (done as u32, ids.len() as u32)
            }
            KIND_ANY_QUEST => {
                let mut ids = listed;
                (ids.any(|id| self.quest_completed(id)) as u32, 1)
            }
            _ if self.has_achievement(rule.id) => (1, 1),
            _ => (0, 0),
        }
    }

    fn quest_completed(&self, quest_id: u32) -> bool {
        let completed = &self.player_data.quest.completed;
        let id = adjust_quest_id(quest_id);
        (id as usize / 8) < completed.data.len() && completed.contains(id)
    }

    /// The level of a class or job, if it has one.
    fn level_for(&self, classjob_id: u16) -> Option<u16> {
        let index = self.game_data.lock().get_exp_array_index(classjob_id)?;
        self.player_data
            .classjob
            .levels
            .0
            .get(index as usize)
            .copied()
    }

    /// Answers the client asking how far along an achievement is.
    pub async fn send_achievement_progress(&mut self, id: u32) {
        let rule = self
            .game_data
            .lock()
            .achievement_rules()
            .iter()
            .find(|rule| rule.id == id)
            .cloned()
            .unwrap_or(AchievementRule {
                id,
                ..Default::default()
            });
        let (current, max) = self.achievement_progress(&rule);
        self.actor_control_self(ActorControlCategory::AchievementProgressInfo { id, current, max })
            .await;
    }

    /// Completes every achievement the character now meets the requirements of (see the top of this file), e.g.
    /// after finishing a quest or levelling up. Achievements made of other achievements are checked again until
    /// nothing more completes.
    /// Returns how many it completed.
    pub async fn check_achievements(&mut self, announce: bool) -> usize {
        let rules: Vec<AchievementRule> = self
            .game_data
            .lock()
            .achievement_rules()
            .iter()
            .filter(|rule| {
                matches!(
                    rule.kind,
                    KIND_ACHIEVEMENTS | KIND_CLASS_LEVEL | KIND_ALL_QUESTS | KIND_ANY_QUEST
                )
            })
            .cloned()
            .collect();

        let mut completed = 0;
        loop {
            let mut completed_any = false;
            for rule in &rules {
                if self.has_achievement(rule.id) {
                    continue;
                }
                let (current, max) = self.achievement_progress(rule);
                if max > 0 && current >= max {
                    self.complete_achievement(rule.id, announce).await;
                    completed_any = true;
                    completed += 1;
                }
            }
            if !completed_any {
                break;
            }
        }
        completed
    }
}
