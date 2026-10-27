//! The Hunting Log: each class's (and Grand Company's) ranks of entries, each a few kinds of enemy to defeat.
//!
//! As retail keeps it (Bob's Chronofoil captures: b23b91ef, 2026-10-26, a new Conjurer's first entries; and every
//! login of his older character, whose logs are at ranks up to 5): twelve logs, by ClassJob's MonsterNote index (0
//! Gladiator ... 7 Arcanist, 8-10 the Grand Companies, 11 Rogue). Each has a rank (0-4; 5 once all are done), and for
//! the rank's ten entries a kill count for each of four targets and a completion bit for each target and each entry.
//! The entries are MonsterNote rows: `<class> * 10000 + rank * 10 + entry + 1` (60002 is the Conjurer's rank 1 entry
//! 2). See docs/src/fork/hunting-log.md.

use serde::{Deserialize, Serialize};

/// How many logs there are (the client's MonsterNoteManager has twelve.)
pub const HUNTING_LOGS: usize = 12;
/// Entries in a rank, and targets in an entry.
pub const ENTRIES: usize = 10;
pub const TARGETS: usize = 4;
/// Ranks in a log; a log at this rank is finished.
pub const RANKS: u8 = 5;
/// The Rogue's log, which retail sends with its unused targets already done (see `fresh`.)
const ROGUE_LOG: u8 = 11;

/// One log's progress.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntingLogProgress {
    /// The log (ClassJob's MonsterNote.)
    pub index: u8,
    /// The rank being worked on (0-4), or `RANKS` when finished.
    pub rank: u8,
    /// Kills of each target of each of the rank's entries (entry * 4 + target.)
    pub counts: Vec<u8>,
    /// Five bits an entry: one for each target done, then one for the entry (entry 1 is bits 5-9.)
    pub completed: u64,
}

/// What a kill did to a log, in the order retail sends it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HuntingLogEvent {
    /// A target was counted (ActorControlSelf 542): the entry's MonsterNote row, the target, the count.
    Counted { note: u32, target: u8, count: u8 },
    /// An entry was completed (543, then its EXP.)
    EntryCompleted { note: u32, reward: u32 },
    /// The rank's every entry is done: the log moved to the next rank (not recorded on retail.)
    RankCompleted { rank: u8 },
}

/// The MonsterNote row of an entry of a class's log.
pub fn note_id(class: u8, rank: u8, entry: usize) -> u32 {
    class as u32 * 10000 + rank as u32 * 10 + entry as u32 + 1
}

/// What the client's log byte 56 holds: a bit per rank, cleared for every rank opened (none while the log is locked;
/// all five once it's finished.) Retail: 0xFFFFFFFE at rank 0, 0xFFFFFFFC at 1, 0xFFFFFFF8 at 2, 0xFFFFFFE0 at 5.
pub fn rank_mask(unlocked: bool, rank: u8) -> u32 {
    if !unlocked {
        return u32::MAX;
    }
    let opened = rank.min(RANKS - 1) + 1;
    !((1u32 << opened) - 1)
}

impl HuntingLogProgress {
    /// A log nothing has been done in. Retail sends the Rogue's with targets 3 and 4 of every entry counted as 0xFF and
    /// done (both its fresh logs in Bob's captures; its entries use at most two targets.)
    pub fn fresh(index: u8) -> Self {
        let mut log = Self {
            index,
            rank: 0,
            counts: vec![0; ENTRIES * TARGETS],
            completed: 0,
        };
        if index == ROGUE_LOG {
            for entry in 0..ENTRIES {
                for target in 2..TARGETS {
                    log.counts[entry * TARGETS + target] = 0xFF;
                    log.completed |= 1 << (entry * 5 + target);
                }
            }
        }
        log
    }

    fn entry_done(&self, entry: usize) -> bool {
        self.completed & (1 << (entry * 5 + 4)) != 0
    }

    /// Counts a defeated enemy (its BNpcName) for the current rank's entries. `entries` gives an entry's targets
    /// (BNpcName, how many; 0 unused) and EXP, by entry. Every unfinished entry with that kind counts it.
    pub fn record_kill(
        &mut self,
        class: u8,
        name_id: u32,
        entries: impl Fn(u32) -> Option<([(u32, u8); TARGETS], u32)>,
    ) -> Vec<HuntingLogEvent> {
        let mut events = Vec::new();
        if self.rank >= RANKS {
            return events;
        }
        self.counts.resize(ENTRIES * TARGETS, 0);
        for entry in 0..ENTRIES {
            if self.entry_done(entry) {
                continue;
            }
            let note = note_id(class, self.rank, entry);
            let Some((targets, reward)) = entries(note) else {
                continue;
            };
            let mut counted = false;
            for (target, (name, needed)) in targets.iter().enumerate() {
                if *needed == 0 || *name != name_id {
                    continue;
                }
                let slot = entry * TARGETS + target;
                if self.counts[slot] >= *needed {
                    continue;
                }
                self.counts[slot] += 1;
                counted = true;
                events.push(HuntingLogEvent::Counted {
                    note,
                    target: target as u8,
                    count: self.counts[slot],
                });
                if self.counts[slot] >= *needed {
                    self.completed |= 1 << (entry * 5 + target);
                }
            }
            let all_done = targets.iter().enumerate().all(|(target, (_, needed))| {
                *needed == 0 || self.counts[entry * TARGETS + target] >= *needed
            });
            if counted && all_done {
                self.completed |= 1 << (entry * 5 + 4);
                events.push(HuntingLogEvent::EntryCompleted { note, reward });
            }
        }
        let rank_exists =
            (0..ENTRIES).any(|entry| entries(note_id(class, self.rank, entry)).is_some());
        if rank_exists
            && (0..ENTRIES).all(|entry| {
                self.entry_done(entry) || entries(note_id(class, self.rank, entry)).is_none()
            })
        {
            self.rank += 1;
            self.counts = vec![0; ENTRIES * TARGETS];
            self.completed = 0;
            events.push(HuntingLogEvent::RankCompleted { rank: self.rank });
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Conjurer's first rank as far as Bob got: entries 1, 2, 4 and 6 (MonsterNote 60001, 60002, 60004, 60006),
    /// one target each, three kills (BNpcNames made up here.)
    fn conjurer(note: u32) -> Option<([(u32, u8); TARGETS], u32)> {
        let one = |name| Some(([(name, 3), (0, 0), (0, 0), (0, 0)], 100));
        match note {
            60001 => one(49),
            60002 => one(37),
            60003..=60010 => one(1000 + note),
            _ => None,
        }
    }

    #[test]
    fn retail_conjurer() {
        let mut log = HuntingLogProgress::fresh(5);
        // Retail (capture b23b91ef): 60002, 60001, 60002, 60002 (done), 60001, 60001 (done)...
        assert_eq!(
            log.record_kill(6, 37, conjurer),
            vec![HuntingLogEvent::Counted {
                note: 60002,
                target: 0,
                count: 1
            }]
        );
        log.record_kill(6, 49, conjurer);
        log.record_kill(6, 37, conjurer);
        assert_eq!(
            log.record_kill(6, 37, conjurer),
            vec![
                HuntingLogEvent::Counted {
                    note: 60002,
                    target: 0,
                    count: 3
                },
                HuntingLogEvent::EntryCompleted {
                    note: 60002,
                    reward: 100
                },
            ]
        );
        assert_eq!(log.completed, 0x220); // retail's flags after 60002
        log.record_kill(6, 49, conjurer);
        log.record_kill(6, 49, conjurer);
        assert_eq!(log.completed, 0x231); // and after 60001
        assert_eq!(&log.counts[..8], &[3, 0, 0, 0, 3, 0, 0, 0]);
        // Done entries don't count again.
        assert!(log.record_kill(6, 37, conjurer).is_empty());
        for _ in 0..3 {
            log.record_kill(6, 1000 + 60004, conjurer);
        }
        assert_eq!(log.completed, 0x88231);
        for _ in 0..3 {
            log.record_kill(6, 1000 + 60006, conjurer);
        }
        assert_eq!(log.completed, 0x22088231); // retail's after 60006
    }

    #[test]
    fn rank_completes() {
        let mut log = HuntingLogProgress::fresh(5);
        let mut last = Vec::new();
        for name in [49, 37]
            .into_iter()
            .chain((60003..=60010).map(|note| 1000 + note))
        {
            for _ in 0..3 {
                last = log.record_kill(6, name, conjurer);
            }
        }
        assert_eq!(
            last.last(),
            Some(&HuntingLogEvent::RankCompleted { rank: 1 })
        );
        assert_eq!((log.rank, log.completed), (1, 0));
    }

    #[test]
    fn masks_and_rogue() {
        assert_eq!(rank_mask(false, 0), 0xFFFF_FFFF);
        assert_eq!(rank_mask(true, 0), 0xFFFF_FFFE);
        assert_eq!(rank_mask(true, 2), 0xFFFF_FFF8);
        assert_eq!(rank_mask(true, 5), 0xFFFF_FFE0);
        // Retail's fresh Rogue log: targets 3 and 4 of every entry 0xFF and done.
        let log = HuntingLogProgress::fresh(11);
        assert_eq!(&log.counts[..4], &[0, 0, 0xFF, 0xFF]);
        assert_eq!(log.completed, 435832222003596);
        assert_eq!(note_id(6, 1, 0), 60011);
    }
}
