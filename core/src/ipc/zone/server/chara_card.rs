use binrw::binrw;
use physis::savedata::chardat::CustomizeData;

/// What an adventurer plate shows (CharaCardData in FFXIVClientStructs, 188 bytes): the portrait and its lighting,
/// the character's look and gear, and the plate's design and profile. The server sends it inside AdventurerPlate, and
/// the client sends it back when the plate is edited (UpdateAdventurerPlate), as the same bytes.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
pub struct CharaCardData {
    pub version: u8,
    pub expression: u8,
    pub camera_zoom: u8,
    pub directional_lighting_color_red: u8,
    pub directional_lighting_color_green: u8,
    pub directional_lighting_color_blue: u8,
    pub directional_lighting_color_brightness: u8,
    pub ambient_lighting_color_red: u8,
    pub ambient_lighting_color_green: u8,
    pub ambient_lighting_color_blue: u8,
    pub ambient_lighting_color_brightness: u8,
    pub class_job_id: u8,
    pub customize: CustomizeData,
    pub stain_ids1: [u8; 12],
    pub gear_visibility_flag: u8,
    pub top_border: u8,
    pub bottom_border: u8,
    pub preferred_class_job_id: u8,
    pub active_hours_weekdays: [u8; 3],
    pub active_hours_weekends: [u8; 3],
    pub play_styles: [u8; 6],
    /// 1: reset by a Fantasia, 2: visible to no one.
    pub flags: u8,
    pub unk13: u8,
    /// 1: friends only.
    pub privacy_flags: u8,
    pub stain_ids2: [u8; 12],
    pub unk14: u8,
    pub banner_timeline: u16,
    pub animation_progress: u16,
    pub head_direction_y: u16,
    pub head_direction_x: u16,
    pub eye_direction_y: u16,
    pub eye_direction_x: u16,
    pub camera_position_x: u16,
    pub camera_position_y: u16,
    pub camera_position_z: u16,
    pub camera_target_x: u16,
    pub camera_target_y: u16,
    pub camera_target_z: u16,
    pub image_rotation: u16,
    pub directional_lighting_vertical: u16,
    pub directional_lighting_horizontal: u16,
    pub banner_decoration: u16,
    pub banner_bg: u16,
    pub banner_frame: u16,
    pub title: u16,
    pub base_plate: u16,
    pub decorations: [u16; 5],
    pub glasses_ids: [u16; 2],
    pub unk18: u32,
    pub item_ids: [u32; 12],
}

impl CharaCardData {
    /// Its size on the wire.
    pub const SIZE: usize = 188;

    /// The bytes the client sent, to save.
    pub fn to_bytes(&self) -> Vec<u8> {
        use binrw::BinWrite;
        let mut cursor = std::io::Cursor::new(Vec::with_capacity(Self::SIZE));
        self.write(&mut cursor).unwrap();
        cursor.into_inner()
    }

    /// Reads saved bytes back, if they're a whole plate.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        use binrw::BinRead;
        if bytes.len() != Self::SIZE {
            return None;
        }
        Self::read(&mut std::io::Cursor::new(bytes)).ok()
    }
}

#[cfg(test)]
mod tests {
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

    use super::*;

    /// The bytes the client sent when Bob saved his plate (2026-10-16 bundle, UpdateAdventurerPlate) read back and
    /// write out unchanged, and the fields land where FFXIVClientStructs says.
    #[test]
    fn round_trips_the_clients_bytes() {
        let mut bytes = vec![0u8; CharaCardData::SIZE];
        bytes[0] = 4; // Version
        bytes[1] = 22; // Expression
        bytes[0x0B] = 6; // ClassJobId
        bytes[0x52] = 5; // BannerTimeline
        bytes[0x52 + 0x24] = 0x53; // TitleId
        bytes[0x52 + 0x26] = 0x01; // BasePlate
        bytes[0x88 + 4] = 0xA8; // ItemIds[0]
        bytes[0x88 + 5] = 0x07;
        let card = CharaCardData::read(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(card.version, 4);
        assert_eq!(card.expression, 22);
        assert_eq!(card.class_job_id, 6);
        assert_eq!(card.banner_timeline, 5);
        assert_eq!(card.title, 0x53);
        assert_eq!(card.base_plate, 1);
        assert_eq!(card.item_ids[0], 0x7A8);

        let mut out = Cursor::new(Vec::new());
        card.write(&mut out).unwrap();
        assert_eq!(out.into_inner(), bytes);
    }
}
