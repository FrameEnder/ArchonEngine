//! Which main scenario quest the Scenario Guide shows: the quest completed last and the ones to do next, worked out
//! from the ScenarioTree's quests and the player's completed ones (`GameData::get_scenario_guide` reads the sheets,
//! `ZoneConnection::send_scenario_guide` sends it). See `docs/src/fork/quests.md` (Scenario Guide).
//!
//! Retail's steps, which the tests below play: A Hero in the Making (511) then the three Grand Companies' The Company
//! You Keep (681, 680, 682), and on to 513 and 3856 (Bob's Chronofoil captures b8432613, c99fe374 and cc62ad94); a
//! split, where two quests are both to be done before the one that joins them, one branch at a time: 986 then 1001
//! alone, 1001 then 1002, 1002 then 3871, 3871 then 1004 (Bob's Chronofoil capture da424662, 2026-11-12).

use std::collections::HashSet;

/// A ScenarioTree quest as the guide sees it (Quest sheet IDs, as the sheet has them.)
#[derive(Debug, Clone, Default)]
pub struct GuideQuest {
    pub id: u32,
    /// Its PreviousQuest (the zeros left out.)
    pub previous: Vec<u32>,
    /// Its QuestLock (the zeros left out): quests that, done, close it.
    pub locks: Vec<u32>,
    /// Where it stands in the story: Expansion, then SortKey.
    pub order: (u8, u16),
    /// What makes it one of a few versions of a step, only one of which a player does: ClassJobRequired,
    /// ClassJobUnlock and GrandCompany (Close to Home per class, The Company You Keep per company.)
    pub variant: (u8, u8, u8),
    /// Between versions of a step, which comes first (the starting class's, then by city; `get_scenario_guide`.)
    pub rank: (bool, usize),
}

/// Whether two quests are versions of one step (only one is done) rather than two branches that are both to be done:
/// one closes the other, or they're for a different class or Grand Company.
fn alternatives(a: &GuideQuest, b: &GuideQuest) -> bool {
    a.locks.contains(&b.id) || b.locks.contains(&a.id) || a.variant != b.variant
}

/// The quest completed last and the ones to do next (up to three), from `quests` (the ScenarioTree's, in the sheet's
/// order), which are `done`, and which are `available` (could be taken up now: their PreviousQuest done, not locked,
/// for this class and company). None when nothing is to be done next.
///
/// What's next after a done quest is what's available after it; when nothing is (a split: the quest after it also
/// waits on the other branch), it's the other branch, the available quests that share a PreviousQuest with it (1001
/// done, 1004 waits on 3871 after 1002: 1002 is next). The quest completed last is the latest done (by `order`) that
/// has something next and none of whose followers is done. Of the quests next, versions of one step are all listed
/// (by `rank`); of branches that are both to be done, retail names only the first (986 then 1001, not 1002 too).
pub fn scenario_guide_step(
    quests: &[GuideQuest],
    done: impl Fn(u32) -> bool,
    available: &HashSet<u32>,
) -> Option<(u32, Vec<u32>)> {
    let followers = |id: u32| {
        quests
            .iter()
            .filter(move |quest| quest.previous.contains(&id))
    };
    let next_after = |quest: &GuideQuest| -> Vec<&GuideQuest> {
        let after: Vec<&GuideQuest> = followers(quest.id)
            .filter(|next| available.contains(&next.id))
            .collect();
        if !after.is_empty() {
            return after;
        }
        quests
            .iter()
            .filter(|other| {
                other.id != quest.id
                    && available.contains(&other.id)
                    && other.previous.iter().any(|id| quest.previous.contains(id))
            })
            .collect()
    };

    let last = quests
        .iter()
        .filter(|quest| done(quest.id))
        .filter(|quest| !followers(quest.id).any(|next| done(next.id)))
        .filter(|quest| !next_after(quest).is_empty())
        .max_by_key(|quest| quest.order)?;

    let mut next = next_after(last);
    // Versions by `rank`; branches both to be done, the earlier in the story first.
    next.sort_by_key(|quest| (quest.rank, quest.order));
    let first = next[0];
    let shown = std::iter::once(first).chain(
        next.iter()
            .skip(1)
            .filter(|quest| alternatives(first, quest))
            .copied(),
    );
    Some((last.id, shown.take(3).map(|quest| quest.id).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quest(id: u32, previous: &[u32], sort: u16) -> GuideQuest {
        GuideQuest {
            id,
            previous: previous.to_vec(),
            order: (0, sort),
            ..Default::default()
        }
    }

    /// Mor Dhona's split (Bob's Chronofoil capture da424662): 986 opens Drowning Out the Voices (1001) and Acting the
    /// Part (1002); 1002 opens Dressed for Conquest (3871); Fool Me Twice (1004) waits on 1001 and 3871. Retail named
    /// one branch at a time. Kawari sent nothing once 1001 was done, and the guide stayed on it (Bob's bundle of
    /// 2026-11-13: "Drowning out the Voices ... didn't remove itself from the Main Scenario Guide").
    #[test]
    fn a_split_one_branch_at_a_time() {
        let quests = vec![
            quest(986, &[984], 1),
            quest(1001, &[986], 2),
            quest(1002, &[986], 3),
            quest(3871, &[1002], 4),
            quest(1004, &[1001, 3871], 5),
        ];
        let step = |done: &[u32]| {
            let is_done = |id: u32| id == 984 || done.contains(&id);
            let available: HashSet<u32> = quests
                .iter()
                .filter(|q| !is_done(q.id) && q.previous.iter().all(|p| is_done(*p)))
                .map(|q| q.id)
                .collect();
            scenario_guide_step(&quests, is_done, &available)
        };
        assert_eq!(step(&[986]), Some((986, vec![1001])));
        assert_eq!(step(&[986, 1001]), Some((1001, vec![1002])));
        assert_eq!(step(&[986, 1001, 1002]), Some((1002, vec![3871])));
        assert_eq!(step(&[986, 1001, 1002, 3871]), Some((3871, vec![1004])));
        // The other way round (not in a capture): Acting the Part first, then its follower and on.
        assert_eq!(step(&[986, 1002]), Some((1002, vec![3871])));
    }

    /// After A Hero in the Making (511), the three Grand Companies' The Company You Keep, each closed by the other two,
    /// all listed (retail's 681, 680, 682: `rank`); after one, that company's enlistment alone.
    #[test]
    fn versions_of_a_step_all_listed() {
        let company = |id: u32, others: [u32; 2], gc: u8, rank: usize| GuideQuest {
            id,
            previous: vec![511],
            locks: others.to_vec(),
            order: (0, 10),
            variant: (0, 0, gc),
            rank: (false, rank),
        };
        let quests = vec![
            quest(511, &[510], 9),
            company(680, [681, 682], 0, 1),
            company(681, [680, 682], 0, 0),
            company(682, [680, 681], 0, 2),
            quest(683, &[681], 11),
        ];
        let done = |id: u32| id == 510 || id == 511;
        let available: HashSet<u32> = [680, 681, 682].into();
        assert_eq!(
            scenario_guide_step(&quests, done, &available),
            Some((511, vec![681, 680, 682]))
        );
        let done = |id: u32| [510, 511, 681].contains(&id);
        assert_eq!(
            scenario_guide_step(&quests, done, &[683].into()),
            Some((681, vec![683]))
        );
    }

    #[test]
    fn nothing_next_nothing_shown() {
        let quests = vec![quest(1, &[], 1), quest(2, &[1], 2)];
        assert_eq!(
            scenario_guide_step(&quests, |_| true, &HashSet::new()),
            None
        );
    }
}
