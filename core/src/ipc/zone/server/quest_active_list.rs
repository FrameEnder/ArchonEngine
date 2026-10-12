use binrw::binrw;

#[binrw]
#[derive(Debug, Clone, Copy, Default)]
/// Matches the client's QuestWork (Application::Network::WorkDefinitions), minus its 8-byte header.
pub struct ActiveQuest {
    pub id: u16,
    pub sequence: u8,
    /// Bit 0 is "priority", bits 1-2 the chosen reward item, bit 3 "hidden".
    pub flags: u8,
    /// The quest variables, UI8A through UI8F.
    pub bitflags: [u8; 6],
    /// The class/job the quest was accepted on.
    #[brw(pad_after = 5)] // padding
    pub accept_class_job: u8,
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

    #[test]
    fn active_quest_layout() {
        use binrw::BinWrite;
        use std::io::Cursor;

        let quest = ActiveQuest {
            id: 124,
            sequence: 1,
            flags: 0,
            bitflags: [0x01, 0x10, 0x10, 0, 0, 0],
            accept_class_job: 6,
        };
        let mut buffer = Cursor::new(Vec::new());
        quest.write_le(&mut buffer).unwrap();
        assert_eq!(
            buffer.into_inner(),
            [124, 0, 1, 0, 0x01, 0x10, 0x10, 0, 0, 0, 6, 0, 0, 0, 0, 0]
        );
    }
}
