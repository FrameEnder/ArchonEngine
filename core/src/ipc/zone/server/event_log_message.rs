#![allow(unused_assignments)] // false positive caused by binrw

use crate::common::HandlerId;
use crate::ipc::zone::server::{ServerZoneIpcData, ServerZoneIpcSegment};
use binrw::binrw;

#[binrw]
#[derive(Debug, Clone, Default)]
#[brw(import{max_params: usize})]
#[brw(assert(params.len() <= max_params, "Too many params! {} > {}", params.len(), max_params))]
pub struct EventLogMessage {
    /// Which event sent this message.
    pub handler_id: HandlerId,
    /// Index into the LogMessage Excel sheet.
    pub message_type: u32,
    #[brw(pad_after = 3)]
    #[br(temp)]
    #[bw(calc = params.len() as u8)]
    params_count: u8,
    #[brw(pad_after = 4)]
    #[br(count = params_count)]
    #[brw(pad_size_to = 4 * max_params)]
    pub params: Vec<u32>,
}

impl EventLogMessage {
    pub fn package(&self) -> Option<ServerZoneIpcSegment> {
        match self.params.len() {
            0..=2 => Some(ServerZoneIpcSegment::new(
                ServerZoneIpcData::EventLogMessage2(self.clone()),
            )),
            3..=4 => Some(ServerZoneIpcSegment::new(
                ServerZoneIpcData::EventLogMessage4(self.clone()),
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::BinWrite;

    use super::*;

    /// A quest's log message as retail sent it (Bob's Kawari Recorder export of 2026-11-12, opcode 478 on 7.58): Allagan
    /// Attire's (1094) LOGMESSAGE0, 3677, after a researcher's talk, no parameters.
    #[test]
    fn a_quest_s_log_message_is_retail_s() {
        let message = EventLogMessage {
            handler_id: HandlerId(0x0001_0446),
            message_type: 3677,
            params: Vec::new(),
        };
        let mut buffer = Cursor::new(Vec::new());
        message
            .write_le_args(&mut buffer, EventLogMessageBinWriteArgs { max_params: 2 })
            .unwrap();
        let mut retail = vec![0x46, 0x04, 0x01, 0x00, 0x5d, 0x0e, 0x00, 0x00];
        retail.resize(24, 0);
        assert_eq!(buffer.into_inner(), retail);
        assert!(matches!(
            message.package().map(|segment| segment.data),
            Some(ServerZoneIpcData::EventLogMessage2(_))
        ));
    }
}
