use binrw::binrw;

use crate::{
    common::{
        EventState, HandlerId, ObjectId, Position, read_bool_from, read_quantized_rotation,
        write_bool_as, write_quantized_rotation,
    },
    ipc::zone::ObjectKind,
};

#[binrw]
#[derive(Debug, Copy, Clone, Default)]
pub struct SpawnObject {
    /// The object's spawn index. Note that this is a completely separate index from actors.
    pub spawn_index: u8,
    /// What kind of object this is.
    pub kind: ObjectKind,
    /// (fork) The byte both of the next two are read from: bit 0 is `not_targetable`, bit 2 `flag_4`. Retail's event
    /// objects had 0, 1, 4 or 5 here (31, 47, 11 and 30 of them in seventeen of Bob's and Connor's Chronofoil
    /// captures); read as one boolean, 5 went out as 1.
    #[br(temp)]
    #[bw(calc = u8::from(*not_targetable) | if *flag_4 { 4 } else { 0 })]
    flags: u8,
    /// Whether this object should be targetable or not.
    #[br(calc = flags & 1 != 0)]
    #[bw(ignore)]
    pub not_targetable: bool,
    /// (fork) Bit 2 of that byte. What it means isn't known; the burning ground Cape Westwind's boss leaves in his
    /// last part (EObj 2012827) has it (Bob's Chronofoil capture 178b0de0 of 2026-11-18.)
    #[br(calc = flags & 4 != 0)]
    #[bw(ignore)]
    pub flag_4: bool,
    /// Whether this object is initially hidden or not.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_hidden: bool,
    /// If this is an ENPC, represents an index into the EObj Excel sheet.
    /// If this is an AreaObject, represents an index into the VFX Excel sheet.
    pub base_id: u32,
    /// Unique ID of this object.
    pub entity_id: ObjectId,
    /// Instance ID of the EventObject in the LGB.
    pub layout_id: u32,
    /// The event handler that owns this object.
    pub handler_id: HandlerId,
    /// The owner actor for this object.
    pub owner_id: ObjectId,
    /// Bound ID of the EventObject in the LGB, usually a SharedGroup.
    pub bind_layout_id: u32,
    /// Radius of the hitbox(?) If `bind_layout_id` is set, this value is ignored.
    /// It seems that the server will send the scale of the EventObject in the layout, if applicable.
    #[brw(pad_after = 2)] // padding for alignment, not read by the client
    pub radius: f32,
    /// The rotation to create the object, in radians.
    #[br(map = read_quantized_rotation)]
    #[bw(map = write_quantized_rotation)]
    pub rotation: f32,
    /// The FATE to associate with.
    pub fate_id: u16,
    /// Controls the animation of the event object.
    #[brw(pad_after = 5)] // padding for alignment, and then an unused u32
    pub event_state: EventState,
    /// For EventObjs, this is the default SharedGroupTimelineState.
    pub args1: u32,
    /// Part of this is used for housing entrances. EventObj uses this too, but it varies based on SubKind.
    pub args2: u32,
    /// The position to place this object at.
    pub position: Position,
}

#[cfg(feature = "server")]
impl mlua::UserData for SpawnObject {}

#[cfg(feature = "server")]
impl mlua::FromLua for SpawnObject {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use super::*;

    /// Retail's SpawnObject for the burning ground Cape Westwind's boss leaves in his last part (Bob's Chronofoil
    /// capture 178b0de0 of 2026-11-18): event object 2012827, its flags byte 5 (not targetable, and bit 2), which
    /// went out as 1 while the byte was one boolean.
    #[test]
    fn the_flags_byte_keeps_both_its_bits() {
        let bytes: [u8; 64] = [
            0x00, 0x07, 0x05, 0x00, 0x9b, 0xb6, 0x1e, 0x00, 0x5c, 0x73, 0x01, 0x40, 0x00, 0x00,
            0x00, 0x00, 0xc4, 0x13, 0x03, 0x80, 0x00, 0x00, 0x00, 0xe0, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0xff, 0x7f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x50, 0x00, 0x0f, 0xf8, 0x2b, 0xc4,
            0x5a, 0xf7, 0x81, 0x42, 0x01, 0x9d, 0x4b, 0xc4,
        ];
        let object = SpawnObject::read_le(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(object.base_id, 2012827);
        assert!(object.not_targetable);
        assert!(object.flag_4);
        assert_eq!(object.args2, 0x0050_0003);
        assert_eq!(object.radius, 1.0);
        let mut written = Cursor::new(Vec::new());
        object.write_le(&mut written).unwrap();
        assert_eq!(written.into_inner(), bytes);

        // And as before for the rest: targetable is 0, not targetable 1.
        let mut written = Cursor::new(Vec::new());
        SpawnObject {
            not_targetable: true,
            ..Default::default()
        }
        .write_le(&mut written)
        .unwrap();
        assert_eq!(written.into_inner()[2], 1);
    }
}
