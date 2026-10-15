//! Opcodes found while the server runs (see `resources/data/fork_opcodes.yml`).
//!
//! The opcode enums are generated with fixed opcodes. One of the fork's own opcodes that isn't known for this game
//! version gets a placeholder opcode the game never uses instead, and when its real opcode is found (e.g. by reading
//! the game's executable at startup) it's registered here: packets coming in with the real opcode are read as the
//! placeholder, and packets going out with the placeholder are sent with the real opcode.

use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// Where the opcode sits in an IPC segment: right after the 0x14 magic of its header.
pub(crate) const OPCODE_OFFSET_IN_IPC: usize = 2;

struct Remap {
    /// The opcode list, e.g. ClientZoneIpcType.
    kind: &'static str,
    /// The opcode the generated enum uses.
    placeholder: u16,
    /// The opcode the game uses.
    real: u16,
}

static REMAPS: RwLock<Vec<Remap>> = RwLock::new(Vec::new());
static ANY: AtomicBool = AtomicBool::new(false);

/// Uses `real` for the opcode that's `placeholder` in the `kind` opcode list (e.g. ClientZoneIpcType.)
pub fn set_real_opcode(kind: &'static str, placeholder: u16, real: u16) {
    let mut remaps = REMAPS.write().unwrap();
    remaps.retain(|remap| !(remap.kind == kind && remap.placeholder == placeholder));
    remaps.push(Remap {
        kind,
        placeholder,
        real,
    });
    ANY.store(true, Ordering::Release);
}

/// Whether any opcode has been found while running.
pub(crate) fn active() -> bool {
    ANY.load(Ordering::Acquire)
}

/// The opcode list an opcode enum is, e.g. ClientZoneIpcType.
pub(crate) fn kind_of<OpCode>() -> &'static str {
    let name = std::any::type_name::<OpCode>();
    name.rsplit("::").next().unwrap_or(name)
}

/// The opcode to read an incoming `opcode` as.
pub(crate) fn incoming(kind: &str, opcode: u16) -> u16 {
    if !ANY.load(Ordering::Acquire) {
        return opcode;
    }
    REMAPS
        .read()
        .unwrap()
        .iter()
        .find(|remap| remap.kind == kind && remap.real == opcode)
        .map(|remap| remap.placeholder)
        .unwrap_or(opcode)
}

/// The opcode to send an outgoing `opcode` as.
pub(crate) fn outgoing(kind: &str, opcode: u16) -> u16 {
    if !ANY.load(Ordering::Acquire) {
        return opcode;
    }
    REMAPS
        .read()
        .unwrap()
        .iter()
        .find(|remap| remap.kind == kind && remap.placeholder == opcode)
        .map(|remap| remap.real)
        .unwrap_or(opcode)
}

/// Rewrites the opcode of an IPC segment (starting at its 0x14 magic) with `map`.
pub(crate) fn rewrite_opcode(ipc: &mut [u8], kind: &str, map: fn(&str, u16) -> u16) {
    if !ANY.load(Ordering::Acquire) || ipc.len() < OPCODE_OFFSET_IN_IPC + 2 {
        return;
    }
    let at = OPCODE_OFFSET_IN_IPC;
    let opcode = u16::from_le_bytes([ipc[at], ipc[at + 1]]);
    let mapped = map(kind, opcode);
    if mapped != opcode {
        ipc[at..at + 2].copy_from_slice(&mapped.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaps_both_ways() {
        set_real_opcode("TestIpcType", 65535, 856);
        assert_eq!(incoming("TestIpcType", 856), 65535);
        assert_eq!(outgoing("TestIpcType", 65535), 856);
        // Other lists and opcodes are left alone.
        assert_eq!(incoming("OtherIpcType", 856), 856);
        assert_eq!(incoming("TestIpcType", 857), 857);

        let mut ipc = vec![0x14, 0x00, 0x58, 0x03, 0xAA];
        rewrite_opcode(&mut ipc, "TestIpcType", incoming);
        assert_eq!(&ipc[2..4], &65535u16.to_le_bytes());
        rewrite_opcode(&mut ipc, "TestIpcType", outgoing);
        assert_eq!(&ipc[2..4], &856u16.to_le_bytes());
    }

    /// A client packet that arrives with an opcode read from the game is read as the packet it stands for.
    #[test]
    fn packets_use_the_real_opcode() {
        use crate::ipc::zone::{ClientZoneIpcData, ClientZoneIpcSegment};
        use crate::opcodes::FORK_OPCODES;
        use crate::packet::{ConnectionState, PacketSegment, SegmentData, SegmentType};
        use binrw::{BinRead, BinWrite};

        // Pretend EventReturnHandler16 was read from the game as an opcode nothing else uses.
        let fork = FORK_OPCODES
            .iter()
            .find(|opcode| opcode.name == "EventReturnHandler16")
            .unwrap();
        let real = 4321;
        assert!(!crate::opcodes::is_known_opcode("ClientZoneIpcType", real));
        set_real_opcode("ClientZoneIpcType", fork.used_opcode, real);

        let state = ConnectionState::None;
        let segment = PacketSegment {
            segment_type: SegmentType::Ipc,
            data: SegmentData::Ipc(ClientZoneIpcSegment::new(
                ClientZoneIpcData::EventReturnHandler16(Default::default()),
            )),
            ..Default::default()
        };
        let mut bytes = std::io::Cursor::new(Vec::new());
        segment.write_le_args(&mut bytes, (&state,)).unwrap();
        let mut bytes = bytes.into_inner();

        // What the game would send: the real opcode.
        let at = crate::packet::PACKET_SEGMENT_HEADER_SIZE as usize + OPCODE_OFFSET_IN_IPC;
        assert_eq!(u16::from_le_bytes([bytes[at], bytes[at + 1]]), fork.used_opcode);
        bytes[at..at + 2].copy_from_slice(&real.to_le_bytes());

        let read = PacketSegment::<ClientZoneIpcSegment>::read_le_args(
            &mut std::io::Cursor::new(&bytes),
            (&state,),
        )
        .unwrap();
        let SegmentData::Ipc(ipc) = read.data else {
            panic!("not an IPC segment");
        };
        assert!(matches!(ipc.data, ClientZoneIpcData::EventReturnHandler16(_)));
    }

    #[test]
    fn kind_is_the_type_name() {
        assert_eq!(kind_of::<crate::opcodes::ClientZoneIpcType>(), "ClientZoneIpcType");
    }
}
