#![allow(unused_assignments)] // false positive caused by binrw

use binrw::binrw;

use crate::common::HandlerId;

#[derive(Debug, Clone, Default)]
#[binrw]
#[brw(import{max_params: usize})]
pub struct MapMarkers {
    /// How many markers to update.
    marker_count: u32,

    /// Icons to set.
    #[br(count = max_params)]
    #[brw(pad_size_to = 4 * max_params)]
    icon_ids: Vec<u32>,

    /// The instance ID in the level.
    #[br(count = max_params)]
    #[brw(pad_size_to = 4 * max_params)]
    layout_ids: Vec<u32>,

    /// The event ID to update for, usually a quest ID.
    #[br(count = max_params)]
    #[brw(pad_size_to = 4 * max_params)]
    handler_ids: Vec<HandlerId>,

    /// Unknown (elevation?)
    #[brw(pad_after = if max_params > 4 { 4 } else { 0 })] // empty
    #[br(count = max_params)]
    #[brw(pad_size_to = max_params)]
    unk: Vec<u8>,
}

impl MapMarkers {
    /// Markers as (icon, layout ID, handler ID), at most as many as the packet holds (`UpdateMapMarkers4` to `32`):
    /// retail's quest givers' markers (Bob's Chronofoil captures: the map icon, the issuer's layout ID, the quest's
    /// event ID, and 0.)
    pub fn new(markers: &[(u32, u32, u32)]) -> Self {
        Self {
            marker_count: markers.len() as u32,
            icon_ids: markers.iter().map(|x| x.0).collect(),
            layout_ids: markers.iter().map(|x| x.1).collect(),
            handler_ids: markers.iter().map(|x| HandlerId(x.2)).collect(),
            unk: vec![0; markers.len()],
        }
    }
}

#[cfg(test)]
mod tests {
    use binrw::BinWrite;
    use std::io::Cursor;

    use super::*;

    /// Retail's UpdateMapMarkers4 in Western Thanalan (Bob's Chronofoil capture da424662): four quests' markers.
    #[test]
    fn writes_as_retail() {
        let markers = MapMarkers::new(&[
            (71021, 4250530, 0x102DF),
            (71021, 4260190, 0x1035A),
            (71021, 4260199, 0x10362),
            (71021, 4260200, 0x10363),
        ]);
        let mut buffer = Cursor::new(Vec::new());
        markers
            .write_le_args(&mut buffer, binrw::args! { max_params: 4 })
            .unwrap();
        let retail = "040000006d1501006d1501006d1501006d150100a2db40005e0141006701410068014100df0201005a030100620301006303010000000000";
        let written: String = buffer
            .into_inner()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(written, retail);
    }
}
