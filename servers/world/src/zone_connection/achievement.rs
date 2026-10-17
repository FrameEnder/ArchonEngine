//! Achievements: the completed ones are saved per character (`unlock.achievements`, newest five in
//! `unlock.achievement_history`), sent when the client asks (the Achievements window), and completed when the
//! character meets what the Achievement sheet asks for.
//!
//! What each Achievement sheet Type asks for, from what the sheet's Key and Data hold for that Type's achievements and
//! when retail completed them (Bob's Chronofoil captures, 2026-10-17). For the list Types the list starts in Key and
//! goes on in Data (e.g. A Life of Adventure I: Key 44, Data 55; Gone from Gridania: Key 66043, Data empty). Only these
//! are checked; the rest can only be given with `!achievement`:
//! * 1: a counter (Key) reaching Data[0], for the counters in `COUNTED` (the rest aren't counted yet)
//! * 2: every achievement in Key and Data
//! * 3: the class or job Key at level Data[0] or above
//! * 6: every quest in Key and Data
//! * 8: map Key unlocked, "Mapping the Realm": retail completes it with the first part discovered on entering the zone
//! * 9: any quest in Key and Data
//! * 20: every aether current in a zone (AetherCurrentCompFlgSet Key), "Freebird"

use kawari::{
    constants::{ACHIEVEMENT_BITMASK_SIZE, ACHIEVEMENT_HISTORY_SIZE},
    ipc::zone::{ActorControlCategory, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    AchievementCounter, ZoneConnection, common::adjust_quest_id, gamedata::AchievementRule,
};

/// The highest achievement the client's bitmap has room for (it ignores higher ones.)
const MAX_ACHIEVEMENT_ID: u32 = ACHIEVEMENT_BITMASK_SIZE as u32 * 8 - 1;

const KIND_COUNTER: u8 = 1;
const KIND_ACHIEVEMENTS: u8 = 2;
const KIND_CLASS_LEVEL: u8 = 3;
const KIND_ALL_QUESTS: u8 = 6;
const KIND_MAP: u8 = 8;
const KIND_ANY_QUEST: u8 = 9;
const KIND_AETHER_CURRENTS: u8 = 20;

/// "Defeat N enemies" (To Crush Your Enemies): an enemy the player had enmity with died, as quests count kills.
pub const COUNTER_ENEMIES_DEFEATED: u32 = 11;

/// "Complete N unique quests" (Storyteller): the quests completed, counted from the character's completed quests, so
/// it's right from the start, as retail's.
pub const COUNTER_UNIQUE_QUESTS: u32 = 214;

/// The Type 1 counters the server counts. Retail's have counted since the character was made; the ones kept here
/// (all but the unique quests) count from when the fork started counting them.
const COUNTED: &[u32] = &[COUNTER_ENEMIES_DEFEATED, COUNTER_UNIQUE_QUESTS];

/// The IDs a list Type (2, 6, 9) asks for: Key, then Data (A Life of Adventure I: Key 44, Data 55; Gone from Gridania:
/// Key 66043, Data empty.)
fn listed_ids(rule: &AchievementRule) -> Vec<u32> {
    std::iter::once(&rule.key)
        .chain(rule.data.iter())
        .filter(|x| **x > 0)
        .map(|x| *x as u32)
        .collect()
}

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

    /// A counter's value (see `COUNTED`.)
    fn achievement_counter(&self, key: u32) -> u32 {
        if key == COUNTER_UNIQUE_QUESTS {
            return self
                .player_data
                .quest
                .completed
                .data
                .iter()
                .map(|byte| byte.count_ones())
                .sum();
        }
        let counters = &self.player_data.unlock.achievement_counters.0;
        counters
            .iter()
            .find(|counter| counter.key == key)
            .map(|counter| counter.value)
            .unwrap_or(0)
    }

    /// Adds to one of the counters "do this N times" achievements count, and completes any that now reach theirs.
    pub async fn add_to_achievement_counter(&mut self, key: u32, amount: u32) {
        let counters = &mut self.player_data.unlock.achievement_counters.0;
        match counters.iter_mut().find(|counter| counter.key == key) {
            Some(counter) => counter.value = counter.value.saturating_add(amount),
            None => counters.push(AchievementCounter { key, value: amount }),
        }
        self.check_achievements(true).await;
    }

    /// How far along an achievement is, as (current, max), for the kinds that are checked; 1 of 1 for one that's
    /// complete and 0 of 0 for the rest, which the client shows without a count.
    fn achievement_progress(&self, rule: &AchievementRule) -> (u32, u32) {
        let listed = listed_ids(rule).into_iter();
        match rule.kind {
            KIND_COUNTER if COUNTED.contains(&(rule.key.max(0) as u32)) => {
                let wanted = rule.data[0].max(0) as u32;
                (
                    self.achievement_counter(rule.key as u32).min(wanted),
                    wanted,
                )
            }
            KIND_MAP => {
                let all = self
                    .game_data
                    .lock()
                    .get_map_discovery_flag(rule.key.max(0) as u32)
                    .unwrap_or(0);
                // Retail completed Mapping the Realm: Central Shroud on entering it for the first time, with its first
                // part discovered (2026-10-14 capture), not once every part was.
                let found = self.discovered_map_parts(rule.key.max(0) as u32) & all;
                ((all != 0 && found != 0) as u32, 1)
            }
            KIND_AETHER_CURRENTS => {
                let flags = &self.player_data.aether_current.comp_flg_set;
                let index = rule.key.max(1) as u32 - 1;
                let done = rule.key > 0
                    && (index as usize / 8) < flags.data.len()
                    && flags.contains(index);
                (done as u32, 1)
            }
            KIND_ACHIEVEMENTS => {
                let ids: Vec<u32> = listed.collect();
                let done = ids.iter().filter(|id| self.has_achievement(**id)).count();
                (done as u32, ids.len() as u32)
            }
            KIND_CLASS_LEVEL => {
                let level = self.level_for(rule.key.max(0) as u16).unwrap_or(0) as u32;
                let wanted = rule.data[0].max(0) as u32;
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
        // It runs while the client's packets wait, so it has to be quick: taking seconds desyncs the connection
        // (2026-10-16, when it read the Map sheet for every Mapping the Realm achievement.)
        let started = std::time::Instant::now();
        let rules: Vec<AchievementRule> = self
            .game_data
            .lock()
            .achievement_rules()
            .iter()
            .filter(|rule| {
                matches!(
                    rule.kind,
                    KIND_ACHIEVEMENTS
                        | KIND_CLASS_LEVEL
                        | KIND_ALL_QUESTS
                        | KIND_ANY_QUEST
                        | KIND_MAP
                        | KIND_AETHER_CURRENTS
                ) || (rule.kind == KIND_COUNTER && COUNTED.contains(&(rule.key.max(0) as u32)))
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
        if started.elapsed() > std::time::Duration::from_millis(50) {
            tracing::warn!("Checking achievements took {:?}", started.elapsed());
        }
        completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: u32, kind: u8, key: i32, data: &[i32]) -> AchievementRule {
        let mut all = [0; 8];
        all[..data.len()].copy_from_slice(data);
        AchievementRule {
            id,
            kind,
            key,
            data: all,
        }
    }

    // Rows of the Achievement sheet as they are (XIVAPI, 2026-10-17).
    #[test]
    fn lists_start_in_key() {
        // A Life of Adventure I: achievements 44 and 55.
        assert_eq!(
            listed_ids(&rule(56, KIND_ACHIEVEMENTS, 44, &[55])),
            vec![44, 55]
        );
        // Gone from Gridania: The Gridanian Envoy (66043), Data empty. Reading only Data, it could never complete.
        assert_eq!(
            listed_ids(&rule(311, KIND_ALL_QUESTS, 66043, &[])),
            vec![66043]
        );
        assert_eq!(adjust_quest_id(66043), 507);
        // Warrior of Light: either of two quests.
        assert_eq!(
            listed_ids(&rule(788, KIND_ANY_QUEST, 66060, &[70058])),
            vec![66060, 70058]
        );
    }

    #[test]
    fn class_level_is_key_and_data0() {
        // I Got the Magic Stick I: conjurer (ClassJob 6) at level 10, in Key and Data[0].
        let magic_stick = rule(45, KIND_CLASS_LEVEL, 6, &[10]);
        assert_eq!((magic_stick.key, magic_stick.data[0]), (6, 10));
    }
}
