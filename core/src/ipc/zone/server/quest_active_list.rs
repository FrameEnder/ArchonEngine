use binrw::binrw;

#[binrw]
#[derive(Debug, Clone, Copy, Default)]
/// Laid out like the client's QuestWork (Application::Network::WorkDefinitions), without its 8-byte header.
pub struct ActiveQuest {
    pub id: u16,
    pub sequence: u8,
    pub flags: u8,
    /// The quest's variables, UI8A through UI8F.
    pub bitflags: [u8; 6],
    /// The class/job the quest was accepted as.
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
}
