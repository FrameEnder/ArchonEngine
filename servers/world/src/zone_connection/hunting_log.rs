//! The Hunting Log as the client is told of it: all twelve logs at login, and after a kill that counts, ActorControlSelf
//! 542 for each target counted, 543 and the entry's EXP for each entry completed, then the log (Bob's Chronofoil
//! capture b23b91ef). The rules are `crate::hunting_log`; see docs/src/fork/hunting-log.md.

use kawari::ipc::zone::{ActorControlCategory, ServerZoneIpcData, ServerZoneIpcSegment};

use crate::{
    ZoneConnection,
    hunting_log::{ENTRIES, HUNTING_LOGS, HuntingLogEvent, HuntingLogProgress, TARGETS, rank_mask},
};

/// The unlock link the Hunting Log is behind: each class's first quest gives it (Way of the Conjurer's SystemReward.)
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

    /// Whether a log is open: the Hunting Log is unlocked and the log's class has been taken up. The Grand Companies'
    /// logs (no class) stay locked for now.
    fn hunting_log_open(&self, index: u8) -> Option<u8> {
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
        (level > 0).then_some(class)
    }

    async fn send_hunting_log(&mut self, log: &HuntingLogProgress, at_login: bool) {
        let open = self.hunting_log_open(log.index).is_some();
        let mut counts = [0; ENTRIES * TARGETS];
        for (slot, count) in counts.iter_mut().zip(log.counts.iter()) {
            *slot = *count;
        }
        let mut mask = rank_mask(open, log.rank);
        // Retail's Rogue log has only the mask's low byte (both of Bob's characters, every login.)
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

    /// A defeated enemy (its BNpcName) counts for the current class's log, if it's open and the enemy is one of its
    /// current rank's targets.
    pub async fn record_hunting_log_kill(&mut self, name_id: u32) {
        let index = {
            let game_data = self.game_data.lock();
            game_data.hunting_log_index(self.player_data.classjob.current_class as u8)
        };
        let Some(index) = index else {
            return;
        };
        let Some(class) = self.hunting_log_open(index) else {
            return;
        };
        let mut log = self.hunting_log(index);
        let events = {
            let game_data = self.game_data.clone();
            log.record_kill(class, name_id, |note| {
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
                        class: class as u32,
                    })
                    .await;
                    if reward > 0 {
                        self.add_exp(reward as i32).await;
                    }
                }
                HuntingLogEvent::RankCompleted { rank } => {
                    tracing::info!(
                        "Hunting Log {index}: rank {rank} reached (retail's packets for it not recorded)"
                    );
                }
            }
        }
        self.send_hunting_log(&log, false).await;
        self.save_hunting_log(log);
    }
}
