#![allow(unused_assignments)] // false positive caused by binrw

use binrw::binrw;

use crate::common::HandlerId;
use crate::ipc::zone::server::{ServerZoneIpcData, ServerZoneIpcSegment};

/// A notice from an event, e.g. a quest's progress pop-up ("Defeated 2/4") as an enemy is killed or an item is found.
///
/// From a retail recording (2026.10.06): quests send notice 1 with [count, needed] for each kill, and notice 0 with no
/// arguments when the sequence moves on. Like EventScene, it comes in sizes by how many arguments it can hold, which
/// all share one handler in the client.
#[binrw]
#[derive(Debug, Clone, Default)]
#[brw(import{max_args: usize})]
#[brw(assert(args.len() <= max_args, "Too many args! {} > {}", args.len(), max_args))]
pub struct EventNotice {
    /// The event (e.g. quest) the notice is from.
    pub handler_id: HandlerId,
    /// Which of the event's notices.
    pub notice_id: u8,
    #[brw(pad_after = 2)]
    #[br(temp)]
    #[bw(calc = args.len() as u8)]
    args_count: u8,
    #[br(count = args_count)]
    #[brw(pad_size_to = 4 * max_args)]
    pub args: Vec<u32>,
}

impl EventNotice {
    /// Packs the notice into the smallest packet that holds its arguments, if it isn't too big.
    pub fn package(&self) -> Option<ServerZoneIpcSegment> {
        match self.args.len() {
            0..=2 => Some(ServerZoneIpcSegment::new(ServerZoneIpcData::EventNotice2(
                self.clone(),
            ))),
            3..=4 => Some(ServerZoneIpcSegment::new(ServerZoneIpcData::EventNotice4(
                self.clone(),
            ))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use super::*;

    /// As retail sent it: quest 376 (You Shall Not Trespass), notice 1, 3 of 4 defeated.
    #[test]
    fn matches_retail() {
        let bytes = [
            0x78, 0x01, 0x01, 0x00, 0x01, 0x02, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00,
        ];
        let notice =
            EventNotice::read_le_args(&mut Cursor::new(&bytes), binrw::args! { max_args: 2 })
                .unwrap();
        assert_eq!(notice.handler_id.0, 0x10178);
        assert_eq!(notice.notice_id, 1);
        assert_eq!(notice.args, vec![3, 4]);

        let mut written = Cursor::new(Vec::new());
        notice
            .write_le_args(&mut written, binrw::args! { max_args: 2 })
            .unwrap();
        assert_eq!(written.into_inner(), bytes);
    }
}
