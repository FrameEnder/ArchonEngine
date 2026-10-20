//! Opcodes found while the server runs (see `resources/data/fork_opcodes.yml`).
//!
//! The opcode enums are generated with fixed opcodes. One of the fork's own opcodes that isn't known for this game
//! version gets a placeholder opcode the game never uses instead, and when its real opcode is found (e.g. by reading
//! the game's executable at startup) it's registered here: packets coming in with the real opcode are read as the
//! placeholder, and packets going out with the placeholder are sent with the real opcode.
//!
//! After a game update a whole opcode list can be remapped at once (`set_table`, from `opcode_match`): then an
//! incoming opcode that isn't in the table can't be some other packet's old opcode, so it's read as unknown (moved to
//! `UNKNOWN_BASE` + opcode, see `real_unknown`), and a packet whose opcode wasn't found goes out as `SWITCHED_OFF`,
//! which the game doesn't use.

use std::collections::HashMap;
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

/// A whole opcode list's new opcodes: the generated enum's opcode → the game's, and back.
struct Table {
    kind: &'static str,
    to_game: HashMap<u16, u16>,
    from_game: HashMap<u16, u16>,
}

static TABLES: RwLock<Vec<Table>> = RwLock::new(Vec::new());

/// Unknown incoming opcodes of a remapped list are read as this plus the opcode (opcodes are below 0x400, so these
/// stay clear of the fork's placeholders, 0xFFxx.)
pub const UNKNOWN_BASE: u16 = 0xF000;
const UNKNOWN_SPAN: u16 = 0x400;
/// Outgoing packets of a remapped list whose opcode wasn't found are sent with this.
pub const SWITCHED_OFF: u16 = 0xFFFE;

/// The game's opcode of an unknown packet as read (undoing `UNKNOWN_BASE`.)
pub fn real_unknown(opcode: u16) -> u16 {
    if (UNKNOWN_BASE..UNKNOWN_BASE + UNKNOWN_SPAN).contains(&opcode) {
        opcode - UNKNOWN_BASE
    } else {
        opcode
    }
}

/// Uses these opcodes for the whole `kind` list: (the generated enum's opcode, the game's.) Opcodes left out are
/// switched off going out; incoming opcodes not in it are read as unknown.
pub fn set_table(kind: &'static str, pairs: &[(u16, u16)]) {
    let mut tables = TABLES.write().unwrap();
    tables.retain(|table| table.kind != kind);
    tables.push(Table {
        kind,
        to_game: pairs.iter().copied().collect(),
        from_game: pairs.iter().map(|(ours, game)| (*game, *ours)).collect(),
    });
    ANY.store(true, Ordering::Release);
}

/// Adds one opcode to a remapped list (e.g. learnt from the size of an unknown packet.)
pub fn add_to_table(kind: &str, ours: u16, game: u16) -> bool {
    let mut tables = TABLES.write().unwrap();
    let Some(table) = tables.iter_mut().find(|table| table.kind == kind) else {
        return false;
    };
    if table.from_game.contains_key(&game) {
        return false;
    }
    table.to_game.insert(ours, game);
    table.from_game.insert(game, ours);
    true
}

/// Whether `kind` has been remapped as a whole.
pub fn has_table(kind: &str) -> bool {
    TABLES
        .read()
        .unwrap()
        .iter()
        .any(|table| table.kind == kind)
}

/// The game's opcode for the generated enum's `ours`, if the list is remapped and it was found.
pub fn table_opcode(kind: &str, ours: u16) -> Option<u16> {
    TABLES
        .read()
        .unwrap()
        .iter()
        .find(|table| table.kind == kind)
        .and_then(|table| table.to_game.get(&ours).copied())
}

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
    if let Some(table) = TABLES
        .read()
        .unwrap()
        .iter()
        .find(|table| table.kind == kind)
    {
        return match table.from_game.get(&opcode) {
            Some(ours) => *ours,
            None if opcode < UNKNOWN_SPAN => UNKNOWN_BASE + opcode,
            None => opcode,
        };
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
    if let Some(table) = TABLES
        .read()
        .unwrap()
        .iter()
        .find(|table| table.kind == kind)
    {
        return table.to_game.get(&opcode).copied().unwrap_or(SWITCHED_OFF);
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
    fn whole_tables() {
        // Two packets that swapped opcodes, one that wasn't found.
        set_table("TableIpcType", &[(100, 200), (200, 100)]);
        assert_eq!(incoming("TableIpcType", 200), 100);
        assert_eq!(incoming("TableIpcType", 100), 200);
        assert_eq!(outgoing("TableIpcType", 100), 200);
        // Not in the table: unknown going in (not read as the packet that had it before), switched off going out.
        assert_eq!(incoming("TableIpcType", 300), UNKNOWN_BASE + 300);
        assert_eq!(real_unknown(incoming("TableIpcType", 300)), 300);
        assert_eq!(outgoing("TableIpcType", 300), SWITCHED_OFF);
        // Learnt later.
        assert!(add_to_table("TableIpcType", 300, 301));
        assert_eq!(incoming("TableIpcType", 301), 300);
        assert!(!add_to_table("TableIpcType", 400, 301));
        assert!(has_table("TableIpcType") && !has_table("OtherTableIpcType"));
    }

    #[test]
    fn kind_is_the_type_name() {
        assert_eq!(kind_of::<crate::opcodes::ClientZoneIpcType>(), "ClientZoneIpcType");
    }
}
