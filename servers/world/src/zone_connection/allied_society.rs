//! Allied societies (the BeastTribe sheet: the Sylphs are row 2): the rank the character has with each, saved in
//! `unlock.allied_society_ranks` (a byte a society, by its row) and sent in PlayerSetup's `beast_reputation_rank`.
//!
//! What retail did as one was met (Bob's Chronofoil capture 7a4214b7, 2026-11-17: Voyce of Concern, quest 1253,
//! rewarded at 4585.9 s): after the gil and before the map markers and the journal, ActorControlSelf 520
//! `[2, 1]` (the Sylphs at rank 1), then the achievement that asks for it (Sylph-conscious I, 867: Achievement Type
//! 15, Key 2, Data 1; ActorControlSelf 515 and ActorControl 518.) A Bad Bladder (1486, the Ixal's first quest)
//! gave no rank in the same capture.
//!
//! Reputation, the daily quests and their allowances aren't here: no capture has one.

use kawari::{constants::BEAST_TRIBE_ARRAY_SIZE, ipc::zone::ActorControlCategory};

use crate::ZoneConnection;

/// The quests that gave a society's first rank on retail, by quest (short ID): the society and the rank
/// (ActorControlSelf 520 as each was rewarded.) Voyce of Concern: capture 7a4214b7.
const QUEST_SOCIETY_RANKS: [(u32, u8, u8); 1] = [(1253, 2, 1)];

/// The Active Help retail answered with ActorControlSelf 524 `[1]` (the allied societies' own, HOWTO_BANZOKU in Voyce
/// of Concern's script, shown in its last scene; the same capture, 4585.5 s.)
const ALLIED_SOCIETY_HELP: u32 = 152;

/// The society and rank a quest gives as it's rewarded (`QUEST_SOCIETY_RANKS`.)
pub fn quest_society_rank(quest: u32) -> Option<(u8, u8)> {
    QUEST_SOCIETY_RANKS
        .iter()
        .find(|(id, _, _)| *id == quest)
        .map(|(_, society, rank)| (*society, *rank))
}

impl ZoneConnection {
    /// The character's rank with a society (0: not met.)
    pub fn allied_society_rank(&self, society: u8) -> u8 {
        self.player_data
            .unlock
            .allied_society_ranks
            .data
            .get(society as usize)
            .copied()
            .unwrap_or(0)
    }

    /// Every society's rank, as PlayerSetup carries them.
    pub fn allied_society_ranks(&self) -> [u8; BEAST_TRIBE_ARRAY_SIZE] {
        let mut ranks = [0; BEAST_TRIBE_ARRAY_SIZE];
        for (society, rank) in ranks.iter_mut().enumerate() {
            *rank = self.allied_society_rank(society as u8);
        }
        ranks
    }

    /// The rank a quest gives with a society, as it's rewarded: saved, told to the client (ActorControlSelf 520), then
    /// the achievements asking for it. Nothing when the character has that rank or a higher one already.
    pub async fn grant_quest_society_rank(&mut self, quest: u32) {
        let Some((society, rank)) = quest_society_rank(quest) else {
            return;
        };
        if self.allied_society_rank(society) >= rank {
            return;
        }
        tracing::info!("Quest {quest} gives allied society {society} rank {rank}");
        let ranks = &mut self.player_data.unlock.allied_society_ranks.data;
        if ranks.len() <= society as usize {
            ranks.resize(BEAST_TRIBE_ARRAY_SIZE, 0);
        }
        ranks[society as usize] = rank;
        self.actor_control_self(ActorControlCategory::SetAlliedSocietyRank {
            society: society as u32,
            rank: rank as u32,
        })
        .await;
        self.check_achievements(true).await;
    }

    /// The client showed an Active Help: retail answered the allied societies' with ActorControlSelf 524 `[1]`.
    pub async fn active_help_shown(&mut self, id: u32) {
        if id == ALLIED_SOCIETY_HELP {
            self.actor_control_self(ActorControlCategory::UnkAlliedSocietyHelp { unk1: 1 })
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Retail gave the Sylphs' first rank with Voyce of Concern and nothing with A Bad Bladder (Bob's Chronofoil
    /// capture 7a4214b7.)
    #[test]
    fn voyce_of_concern_meets_the_sylphs() {
        assert_eq!(quest_society_rank(1253), Some((2, 1)));
        assert_eq!(quest_society_rank(1486), None);
        assert_eq!(quest_society_rank(1252), None);
    }
}
