//! The Hunting Log as the client is told of it: all twelve logs at login, and after a kill that counts, ActorControlSelf
//! 542 for each target counted, 543 and the entry's reward for each entry completed, then the log (Bob's and Connor's
//! Chronofoil captures). A kill counts for the current class's log and for the player's Grand Company's. The rules
//! are `crate::hunting_log`; see docs/src/fork/hunting-log.md.

use kawari::ipc::zone::{ActorControlCategory, ServerZoneIpcData, ServerZoneIpcSegment};

use crate::{
    ZoneConnection,
    hunting_log::{
        ENTRIES, FIRST_COMPANY_LOG, HUNTING_LOGS, HuntingLogEvent, HuntingLogProgress, TARGETS,
        company_of, note_base, note_group, ranks_of,
    },
};

/// The unlock link the classes' Hunting Logs are behind: each class's first quest gives it (Way of the Conjurer's
/// SystemReward.)
pub const HUNTING_LOG_UNLOCK: u32 = 21;

impl ZoneConnection {
    /// A log's progress (a fresh one if it hasn't been begun.)
    fn hunting_log(&self, index: u8) -> HuntingLogProgress {
        self.player_data
            .unlock
            .hunting_log
            .0
            .iter()
            .find(|log| log.index == index)
            .cloned()
            .unwrap_or_else(|| HuntingLogProgress::fresh(index))
    }

    fn save_hunting_log(&mut self, log: HuntingLogProgress) {
        let logs = &mut self.player_data.unlock.hunting_log.0;
        match logs.iter_mut().find(|saved| saved.index == log.index) {
            Some(saved) => *saved = log,
            None => logs.push(log),
        }
    }

    /// Whether a log can be worked on, and if so where its entries' rows start and how many of its ranks are open.
    /// A class's: the Hunting Log is unlocked (a class's first quest) and the class has been taken up (its level
    /// isn't 0), every rank as the one before is finished: Bob's Conjurer counted Gladiator kills at Gladiator 1 to 4
    /// with no Gladiator quest done, and nothing counted for Connor before his first class quest. A Grand Company's:
    /// the player is in it, a rank at a time by their rank in it (`GameData::company_hunting_log_ranks`).
    fn hunting_log_open(&self, index: u8) -> Option<(u32, u8)> {
        if let Some(company) = company_of(index) {
            if self.player_data.grand_company.active_company as u8 != company {
                return None;
            }
            let rank = self
                .player_data
                .grand_company
                .company_ranks
                .0
                .get(company as usize - 1)
                .copied()
                .unwrap_or_default();
            let open = self.game_data.lock().company_hunting_log_ranks(rank);
            return Some((note_base(index, None)?, open));
        }
        if !self.player_data.unlock.unlocks.contains(HUNTING_LOG_UNLOCK) {
            return None;
        }
        let game_data = self.game_data.lock();
        let class = game_data.hunting_log_class(index)?;
        let exp_index = game_data.get_exp_array_index(class as u16)?;
        let level = self
            .player_data
            .classjob
            .levels
            .0
            .get(exp_index as usize)
            .copied()
            .unwrap_or_default();
        (level > 0).then_some((note_base(index, Some(class))?, ranks_of(index)))
    }

    async fn send_hunting_log(&mut self, log: &HuntingLogProgress, at_login: bool) {
        let mut counts = [0; ENTRIES * TARGETS];
        for (slot, count) in counts.iter_mut().zip(log.counts.iter()) {
            *slot = *count;
        }
        // The ranks not looked at yet. Retail's Rogue log has only the mask's low byte (both of Bob's characters,
        // every login: 0xFF, 0xFE once seen.)
        let mut mask = log.mask();
        if log.index == 11 {
            mask &= 0xFF;
        }
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::HuntingLog {
            unk1: if at_login { -1 } else { 0 },
            rank: log.rank,
            index: log.index,
            counts,
            completed: log.completed,
            rank_mask: mask,
        });
        self.send_ipc_self(ipc).await;
    }

    /// At login: every log, as retail sends them.
    pub async fn send_hunting_logs(&mut self) {
        for index in 0..HUNTING_LOGS as u8 {
            let log = self.hunting_log(index);
            self.send_hunting_log(&log, true).await;
        }
    }

    /// The client says the player looked at a rank of a log (ClientTrigger 401): kept, and shown in the log's last
    /// field from the next time it's sent. Retail answers nothing.
    pub fn view_hunting_log(&mut self, index: u32, rank: u32) {
        if index >= HUNTING_LOGS as u32 {
            return;
        }
        let mut log = self.hunting_log(index as u8);
        if log.view(rank) {
            self.save_hunting_log(log);
        }
    }

    /// A defeated enemy (its BNpcName) counts for the current class's log and for the player's Grand Company's, where
    /// it's one of their current rank's targets.
    pub async fn record_hunting_log_kill(&mut self, name_id: u32) {
        let class_log = {
            let game_data = self.game_data.lock();
            game_data.hunting_log_index(self.player_data.classjob.current_class as u8)
        };
        let company = self.player_data.grand_company.active_company as u8;
        let company_log = (1..=3)
            .contains(&company)
            .then(|| FIRST_COMPANY_LOG + company - 1);
        for index in class_log.into_iter().chain(company_log) {
            self.record_hunting_log_kill_in(index, name_id).await;
        }
    }

    async fn record_hunting_log_kill_in(&mut self, index: u8, name_id: u32) {
        let Some((base, open_ranks)) = self.hunting_log_open(index) else {
            return;
        };
        let company = company_of(index);
        let mut log = self.hunting_log(index);
        let events = {
            let game_data = self.game_data.clone();
            log.record_kill(base, name_id, open_ranks, |note| {
                game_data
                    .lock()
                    .monster_note(note)
                    .map(|entry| (entry.targets, entry.reward))
            })
        };
        if events.is_empty() {
            return;
        }
        for event in &events {
            match *event {
                HuntingLogEvent::Counted {
                    note,
                    target,
                    count,
                } => {
                    self.actor_control_self(ActorControlCategory::HuntingLogTargetCounted {
                        note,
                        target: target as u32,
                        count: count as u32,
                    })
                    .await;
                }
                HuntingLogEvent::EntryCompleted { note, reward } => {
                    self.actor_control_self(ActorControlCategory::HuntingLogEntryCompleted {
                        note,
                        class: note_group(note),
                    })
                    .await;
                    self.give_hunting_log_reward(company, reward).await;
                }
                HuntingLogEvent::RankCompleted { rank } => {
                    // Retail's packets for a finished rank weren't recorded: its bonus as an entry's reward is given.
                    let reward = self
                        .game_data
                        .lock()
                        .hunting_log_rank_reward(rank, company.is_some());
                    tracing::info!(
                        "Hunting Log {index}: rank {} finished, {reward} {}",
                        rank + 1,
                        if company.is_some() { "seals" } else { "EXP" }
                    );
                    self.give_hunting_log_reward(company, reward).await;
                }
            }
        }
        self.send_hunting_log(&log, false).await;
        self.save_hunting_log(log);
    }

    /// An entry's or a rank's reward: EXP for a class's log (ActorControlSelf 7, as any EXP without a source: retail's
    /// after 543), seals for a Grand Company's.
    async fn give_hunting_log_reward(&mut self, company: Option<u8>, reward: u32) {
        if reward == 0 {
            return;
        }
        match company {
            Some(company) => self.give_company_seals(company as u32, reward).await,
            None => self.add_exp(reward as i32).await,
        }
    }
}
