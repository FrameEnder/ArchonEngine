use binrw::binrw;

#[binrw]
#[derive(Debug, Clone, Copy, Default)]
/// Mapped against the client's QuestWork with marker bytes (see FrameEnder/kawari-diagnostics, 2026-10-13).
pub struct ActiveQuest {
    pub id: u16,
    pub sequence: u8,
    pub flags: u8,
    /// The class/job the quest was accepted as.
    #[brw(pad_after = 5)] // the client doesn't keep these
    pub accept_class_job: u8,
    /// The quest's variables, UI8A through UI8F.
    pub bitflags: [u8; 6],
}

impl ActiveQuest {
    pub const SIZE: usize = 16;
}

#[binrw]
#[derive(Debug, Clone, Default)]
pub struct QuestActiveList {
    #[br(count = 30)]
    #[brw(pad_size_to = 30 * ActiveQuest::SIZE)]
    pub quests: Vec<ActiveQuest>,
}

#[cfg(test)]
mod tests {
    use crate::common::ensure_size;

    use super::*;

    #[test]
    fn active_quest_size() {
        ensure_size::<ActiveQuest, { ActiveQuest::SIZE }>();
    }

    /// The client put these marker bytes into QuestWork's Flags, AcceptClassJob and Variables.
    #[test]
    fn active_quest_layout() {
        use binrw::BinWrite;

        let quest = ActiveQuest {
            id: 124,
            sequence: 1,
            flags: 0xA3,
            accept_class_job: 0xA4,
            bitflags: [0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF],
        };
        let mut buffer = std::io::Cursor::new(Vec::new());
        quest.write_le(&mut buffer).unwrap();
        assert_eq!(
            buffer.into_inner(),
            [
                0x7C, 0x00, 0x01, 0xA3, 0xA4, 0, 0, 0, 0, 0, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF
            ]
        );
    }
}
