//! The Hunting Log: each class's (and Grand Company's) ranks of entries, each a few kinds of enemy to defeat.
//!
//! As retail keeps it (Bob's and Connor's Chronofoil captures, 92 logins and 36 kills counted; and the game's sheets):
//! twelve logs, by ClassJob's MonsterNote index (0 Gladiator ... 7 Arcanist, 8-10 the Grand Companies, 11 Rogue). Each
//! has a rank (the ranks finished: 0-4, 5 once all are done; a company's has three), and for the rank's ten entries a
//! kill count for each of four targets and a completion bit for each target and each entry. The entries are
//! MonsterNote rows: a class's start at its ClassJob times 10,000 (60002 is the Conjurer's rank 1 entry 2; the
//! Arcanist's at 260001, the Rogue's at 290001), a company's at its number times 1,000,000 (the Maelstrom's at
//! 1000001). A class's entry gives EXP, a company's seals (MonsterNote's Reward); finishing a rank gives ParamGrow's
//! HuntingLogExpReward (2,500, 10,000, 20,000, 30,000, 40,000) or MonsterNoteSeals (1,181, 2,141, 3,603).
//!
//! The log's last field is a bit per rank the player hasn't looked at yet: all set until the client says a rank was
//! viewed (ClientTrigger 401: the log, the rank), whatever has been counted. Kills count either way.
//! See docs/src/fork/hunting-log.md.

use serde::{Deserialize, Serialize};

/// How many logs there are (the client's MonsterNoteManager has twelve.)
pub const HUNTING_LOGS: usize = 12;
/// Entries in a rank, and targets in an entry.
pub const ENTRIES: usize = 10;
pub const TARGETS: usize = 4;
/// Ranks in a class's log; a log at this rank is finished.
pub const RANKS: u8 = 5;
/// Ranks in a Grand Company's log (30 of its 50 MonsterNote rows are used.)
pub const COMPANY_RANKS: u8 = 3;
/// The Grand Companies' logs: 8 the Maelstrom's, 9 the Twin Adder's, 10 the Immortal Flames'.
pub const FIRST_COMPANY_LOG: u8 = 8;
pub const LAST_COMPANY_LOG: u8 = 10;
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
    /// A bit per rank the player has looked at (the client's ClientTrigger 401.)
    #[serde(default)]
    pub seen: u32,
}

/// What a kill did to a log, in the order retail sends it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HuntingLogEvent {
    /// A target was counted (ActorControlSelf 542): the entry's MonsterNote row, the target, the count.
    Counted { note: u32, target: u8, count: u8 },
    /// An entry was completed (543, then its EXP.)
    EntryCompleted { note: u32, reward: u32 },
    /// The rank's every entry is done (the rank finished, 0-based): its bonus, and the log moves to the next rank.
    /// Retail's packets for it weren't recorded.
    RankCompleted { rank: u8 },
}

/// The Grand Company (1 the Maelstrom, 2 the Twin Adder, 3 the Immortal Flames) a log is for, if it's one's.
pub fn company_of(index: u8) -> Option<u8> {
    (FIRST_COMPANY_LOG..=LAST_COMPANY_LOG)
        .contains(&index)
        .then(|| index - FIRST_COMPANY_LOG + 1)
}

/// How many ranks a log has.
pub fn ranks_of(index: u8) -> u8 {
    if company_of(index).is_some() {
        COMPANY_RANKS
    } else {
        RANKS
    }
}

/// Where a log's MonsterNote rows start: a class's at its ClassJob times 10,000, a company's at its number times
/// 1,000,000 (the sheet: 10001 Gladiator 01, 260001 Arcanist 01, 290001 Rogue 01, 1000001 Maelstrom 01.)
pub fn note_base(index: u8, class: Option<u8>) -> Option<u32> {
    match company_of(index) {
        Some(company) => Some(company as u32 * 1_000_000),
        None => class.map(|class| class as u32 * 10_000),
    }
}

/// The MonsterNote row of an entry of a log (`note_base`).
pub fn note_id(base: u32, rank: u8, entry: usize) -> u32 {
    base + rank as u32 * 10 + entry as u32 + 1
}

/// ActorControlSelf 543's second number for an entry: 1 for the Gladiator's and 6 for the Conjurer's on retail, which
/// is the entry's row over 10,000 (its class; a company's would be 100, 200 or 300, unseen.)
pub fn note_group(note: u32) -> u32 {
    note / 10_000
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
            seen: 0,
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

    /// The log's last field: a bit per rank not yet looked at. Retail: 0xFFFFFFFF until the log is opened, whatever was
    /// counted (Bob's paladin at 45, Connor's Conjurer at 50); 0xFFFFFFFE once its first rank was (401 `[5, 0]`); his
    /// older character's 0xFFFFFFFC, ...F8 and ...E0 for logs at ranks 1, 2 and 5, and 0xFFFFFFFE for a Marauder's
    /// whose second rank was never opened.
    pub fn mask(&self) -> u32 {
        !self.seen
    }

    /// The client says a rank was looked at (ClientTrigger 401). True if that's news.
    pub fn view(&mut self, rank: u32) -> bool {
        if rank >= ranks_of(self.index) as u32 || self.seen & (1 << rank) != 0 {
            return false;
        }
        self.seen |= 1 << rank;
        true
    }

    /// Empties the counts for a new rank (the Rogue's unused targets as retail has them.)
    fn clear_rank(&mut self) {
        let fresh = Self::fresh(self.index);
        self.counts = fresh.counts;
        self.completed = fresh.completed;
    }

    fn entry_done(&self, entry: usize) -> bool {
        self.completed & (1 << (entry * 5 + 4)) != 0
    }

    /// Counts a defeated enemy (its BNpcName) for the current rank's entries. `base` is where the log's rows start
    /// (`note_base`); `open_ranks` how many of its ranks can be worked on (all of a class's; a company's by the
    /// player's rank in it); `entries` gives an entry's targets (BNpcName, how many; 0 unused) and reward, by row.
    /// Every unfinished entry with that kind counts it.
    pub fn record_kill(
        &mut self,
        base: u32,
        name_id: u32,
        open_ranks: u8,
        entries: impl Fn(u32) -> Option<([(u32, u8); TARGETS], u32)>,
    ) -> Vec<HuntingLogEvent> {
        let mut events = Vec::new();
        if self.rank >= ranks_of(self.index).min(open_ranks) {
            return events;
        }
        self.counts.resize(ENTRIES * TARGETS, 0);
        for entry in 0..ENTRIES {
            if self.entry_done(entry) {
                continue;
            }
            let note = note_id(base, self.rank, entry);
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
        let used = |entry: usize| {
            entries(note_id(base, self.rank, entry))
                .is_some_and(|(targets, _)| targets.iter().any(|(_, needed)| *needed != 0))
        };
        let rank_exists = (0..ENTRIES).any(used);
        if !events.is_empty()
            && rank_exists
            && (0..ENTRIES).all(|entry| self.entry_done(entry) || !used(entry))
        {
            events.push(HuntingLogEvent::RankCompleted { rank: self.rank });
            self.rank += 1;
            self.clear_rank();
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
            log.record_kill(60000, 37, 5, conjurer),
            vec![HuntingLogEvent::Counted {
                note: 60002,
                target: 0,
                count: 1
            }]
        );
        log.record_kill(60000, 49, 5, conjurer);
        log.record_kill(60000, 37, 5, conjurer);
        assert_eq!(
            log.record_kill(60000, 37, 5, conjurer),
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
        log.record_kill(60000, 49, 5, conjurer);
        log.record_kill(60000, 49, 5, conjurer);
        assert_eq!(log.completed, 0x231); // and after 60001
        assert_eq!(&log.counts[..8], &[3, 0, 0, 0, 3, 0, 0, 0]);
        // Done entries don't count again.
        assert!(log.record_kill(60000, 37, 5, conjurer).is_empty());
        for _ in 0..3 {
            log.record_kill(60000, 1000 + 60004, 5, conjurer);
        }
        assert_eq!(log.completed, 0x88231);
        for _ in 0..3 {
            log.record_kill(60000, 1000 + 60006, 5, conjurer);
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
                last = log.record_kill(60000, name, 5, conjurer);
            }
        }
        assert_eq!(
            last.last(),
            Some(&HuntingLogEvent::RankCompleted { rank: 0 })
        );
        assert_eq!((log.rank, log.completed), (1, 0));
        // The rank byte moves on though the new rank hasn't been looked at: retail's Marauder at rank 1 with only
        // its first rank seen (0xFFFFFFFE; Bob's older character, every login.)
        log.view(0);
        assert_eq!((log.rank, log.mask()), (1, 0xFFFF_FFFE));
    }

    /// What the client is told of which ranks were looked at (ClientTrigger 401; Bob's Chronofoil captures b23b91ef,
    /// `[5, 0]`: 0xFFFFFFFF then 0xFFFFFFFE; c99fe374, `[10, 0]` for the Immortal Flames' log.)
    #[test]
    fn seen_ranks() {
        let mut log = HuntingLogProgress::fresh(5);
        // Counted kills don't open it: Connor's Conjurer finished entries with the mask still 0xFFFFFFFF.
        for _ in 0..3 {
            log.record_kill(60000, 37, 5, conjurer);
        }
        assert_eq!((log.completed, log.mask()), (0x220, 0xFFFF_FFFF));
        assert!(log.view(0));
        assert_eq!(log.mask(), 0xFFFF_FFFE);
        assert!(!log.view(0), "seen already");
        assert!(log.view(1) && log.view(2));
        assert_eq!(log.mask(), 0xFFFF_FFF8);
        assert!(log.view(4) && log.view(3));
        assert_eq!(log.mask(), 0xFFFF_FFE0); // a finished Thaumaturge's
        assert!(!log.view(5), "no sixth rank");
        // A company's has three.
        let mut flames = HuntingLogProgress::fresh(10);
        assert!(flames.view(0) && flames.view(2) && !flames.view(3));
        // An older save has no `seen`.
        let old: HuntingLogProgress =
            serde_json::from_str(r#"{"index":5,"rank":1,"counts":[],"completed":0}"#).unwrap();
        assert_eq!(old.mask(), 0xFFFF_FFFF);
    }

    /// The Immortal Flames' log (rows from 3000001; the sheet's first: 3 of BNpcName 124's kind, 240 seals), open a
    /// rank at a time by the player's rank in the company.
    #[test]
    fn a_company_log() {
        let flames = |note: u32| -> Option<([(u32, u8); TARGETS], u32)> {
            match note {
                3000001..=3000030 => Some(([(note - 3000000, 1), (0, 0), (0, 0), (0, 0)], 240)),
                3000031..=3000050 => Some(([(0, 0); TARGETS], 0)), // the sheet's unused rows
                _ => None,
            }
        };
        assert_eq!(
            (company_of(10), ranks_of(10), note_base(10, None)),
            (Some(3), 3, Some(3_000_000))
        );
        assert_eq!(
            (company_of(5), ranks_of(5), note_base(5, Some(6))),
            (None, 5, Some(60_000))
        );
        assert_eq!(
            note_base(7, Some(26)).map(|base| note_id(base, 0, 0)),
            Some(260_001)
        ); // Arcanist 01
        assert_eq!(
            note_base(11, Some(29)).map(|base| note_id(base, 4, 9)),
            Some(290_050)
        ); // Rogue 50
        assert_eq!(
            (note_group(10002), note_group(60006), note_group(3000001)),
            (1, 6, 300)
        );

        let mut log = HuntingLogProgress::fresh(10);
        let mut events = Vec::new();
        for name in 1..=10 {
            events = log.record_kill(3_000_000, name, 1, flames);
        }
        assert_eq!(
            events,
            vec![
                HuntingLogEvent::Counted {
                    note: 3000010,
                    target: 0,
                    count: 1
                },
                HuntingLogEvent::EntryCompleted {
                    note: 3000010,
                    reward: 240
                },
                HuntingLogEvent::RankCompleted { rank: 0 },
            ]
        );
        assert_eq!(log.rank, 1);
        // The second rank's kills wait for the player's rank in the company.
        assert!(log.record_kill(3_000_000, 11, 1, flames).is_empty());
        assert_eq!(log.record_kill(3_000_000, 11, 2, flames).len(), 2);
        for name in 12..=30 {
            log.record_kill(3_000_000, name, 3, flames);
        }
        assert_eq!(log.rank, 3, "finished: retail's older character's, rank 3");
        assert!(log.record_kill(3_000_000, 1, 3, flames).is_empty());
    }

    /// A company's log's ranks by the rank in the company (GrandCompanyRank's RequiredHuntingLogRank: 1 on rank 5,
    /// Sergeant Third Class; 2 on rank 9, Second Lieutenant.)
    #[test]
    fn company_ranks_open() {
        use crate::gamedata::company_log_ranks_open;
        let asked = [(5, 1), (9, 2)];
        assert_eq!(company_log_ranks_open(&asked, 1), 1);
        assert_eq!(company_log_ranks_open(&asked, 4), 1);
        assert_eq!(company_log_ranks_open(&asked, 5), 2);
        assert_eq!(company_log_ranks_open(&asked, 8), 2);
        assert_eq!(company_log_ranks_open(&asked, 9), 3);
        assert_eq!(company_log_ranks_open(&asked, 11), 3);
        assert_eq!(
            company_log_ranks_open(&[], 3),
            1,
            "the sheet unread: the first rank"
        );
    }

    #[test]
    fn masks_and_rogue() {
        // Retail's fresh Rogue log: targets 3 and 4 of every entry 0xFF and done.
        let log = HuntingLogProgress::fresh(11);
        assert_eq!(&log.counts[..4], &[0, 0, 0xFF, 0xFF]);
        assert_eq!(log.completed, 435832222003596);
        assert_eq!(note_id(60_000, 1, 0), 60011);
        // A new rank of the Rogue's starts the same way.
        let mut rogue = HuntingLogProgress::fresh(11);
        rogue.rank = 1;
        rogue.counts[0] = 2;
        rogue.clear_rank();
        assert_eq!(
            rogue,
            HuntingLogProgress {
                rank: 1,
                ..HuntingLogProgress::fresh(11)
            }
        );
    }
}
