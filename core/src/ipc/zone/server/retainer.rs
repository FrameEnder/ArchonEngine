//! Retainer packets the fork added (docs/src/fork/retainers.md). The layouts are from Bob's Chronofoil captures of
//! retail on 2026.09.15; FFXIVOpcodes' names for that version's opcodes say which packet is which.

use binrw::binrw;

use crate::common::{CHAR_NAME_MAX_LENGTH, ContainerType, read_string, write_string};

/// `RetainerState::state` when a retainer is called at a bell.
pub const RETAINER_STATE_SUMMONED: u8 = 4;
/// `RetainerState::state` when they're sent away.
pub const RETAINER_STATE_DISMISSED: u8 = 3;

/// A retainer is called or sent away at a summoning bell (FFXIVOpcodes' RetainerState.)
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct RetainerState {
    pub retainer_id: u64,
    pub unk1: u32,
    /// The top byte was 0x01 or 0x81 on retail for a retainer with items on the market, 0 otherwise.
    pub flags: u32,
    pub unk2: u32,
    /// `RETAINER_STATE_SUMMONED` or `RETAINER_STATE_DISMISSED`.
    pub state: u8,
    #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
    #[br(count = CHAR_NAME_MAX_LENGTH)]
    #[br(map = read_string)]
    #[bw(map = write_string)]
    #[brw(pad_after = 3)]
    pub name: String,
}

impl RetainerState {
    pub const SIZE: usize = 56;
}

/// One retainer in `PlayerRetainerInfo`.
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct PlayerRetainerEntry {
    pub retainer_id: u64,
    /// The same for every retainer of a character, and moved on between captures days apart; looks like a Unix time.
    pub unk1: u32,
    /// 1 on retail.
    pub unk2: u8,
    /// 0 or 1 on retail.
    pub unk3: u8,
    /// 5 on retail.
    pub unk4: u8,
    #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
    #[br(count = CHAR_NAME_MAX_LENGTH)]
    #[br(map = read_string)]
    #[bw(map = write_string)]
    #[brw(pad_after = 1)]
    pub name: String,
}

/// How many retainers `PlayerRetainerInfo` holds.
pub const PLAYER_RETAINER_ENTRIES: usize = 10;

/// The character's retainers, for the marketboard's listings (FFXIVOpcodes' PlayerRetainerInfo, FFXIVClientStructs'
/// InfoProxyItemSearch.ProcessPlayerRetainerInfo.) The reply to `ClientZoneIpcData::RequestPlayerRetainerInfo`, which
/// the client sends whenever it opens a retainer's sale list.
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct PlayerRetainerInfo {
    pub entries: [PlayerRetainerEntry; PLAYER_RETAINER_ENTRIES],
}

impl PlayerRetainerInfo {
    pub const SIZE: usize = 480;
}

/// The asking price of one item a retainer has on the market (FFXIVOpcodes' ItemMarketBoardInfo), sent after their
/// containers as they're called.
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct ItemMarketBoardInfo {
    pub sequence: u32,
    #[brw(pad_size_to = 4)]
    pub container: ContainerType,
    #[brw(pad_after = 2)]
    pub slot: u16,
    pub unk: u32,
    #[brw(pad_after = 12)]
    pub price_per_unit: u32,
}

impl ItemMarketBoardInfo {
    pub const SIZE: usize = 32;
}

/// How many items a retainer has on the market, after their `ItemMarketBoardInfo`s (FFXIVOpcodes'
/// ItemMarketBoardSummary.)
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct ItemMarketBoardSummary {
    pub sequence: u32,
    pub count: u32,
}

impl ItemMarketBoardSummary {
    pub const SIZE: usize = 8;
}

/// A retainer's sale history (FFXIVOpcodes' RetainerSaleHistory), the reply to
/// `ClientZoneIpcData::RequestRetainerSaleHistory`. Only the retainer is known: the rest is the sales, which Kawari
/// doesn't keep yet, so it's sent empty.
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct RetainerSaleHistory {
    #[brw(pad_after = 1040)]
    pub retainer_id: u64,
}

impl RetainerSaleHistory {
    pub const SIZE: usize = 1048;
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::BinWrite;

    use crate::common::ensure_size;

    use super::*;

    #[test]
    fn retainer_sizes() {
        ensure_size::<RetainerState, { RetainerState::SIZE }>();
        ensure_size::<PlayerRetainerInfo, { PlayerRetainerInfo::SIZE }>();
        ensure_size::<ItemMarketBoardInfo, { ItemMarketBoardInfo::SIZE }>();
        ensure_size::<ItemMarketBoardSummary, { ItemMarketBoardSummary::SIZE }>();
        ensure_size::<RetainerSaleHistory, { RetainerSaleHistory::SIZE }>();
    }

    /// The bytes of a retail RetainerState as a retainer was called (Bob's 2026-09-15 capture, name replaced.)
    #[test]
    fn retainer_state_layout() {
        let state = RetainerState {
            retainer_id: 0x0078_0017_4aa7_e097,
            state: RETAINER_STATE_SUMMONED,
            name: "Abc".to_string(),
            ..Default::default()
        };
        let mut buffer = Cursor::new(Vec::new());
        state.write_le(&mut buffer).unwrap();
        let bytes = buffer.into_inner();
        assert_eq!(
            &bytes[..8],
            &[0x97, 0xe0, 0xa7, 0x4a, 0x17, 0x00, 0x78, 0x00]
        );
        assert_eq!(&bytes[8..20], &[0; 12]);
        assert_eq!(bytes[20], 4);
        assert_eq!(&bytes[21..24], b"Abc");
    }

    /// A retail ItemMarketBoardInfo: slot 1 of the market container at 30,000 gil.
    #[test]
    fn item_market_board_info_layout() {
        let info = ItemMarketBoardInfo {
            sequence: 0xc0,
            container: ContainerType::RetainerMarket,
            slot: 1,
            unk: 0,
            price_per_unit: 30000,
        };
        let mut buffer = Cursor::new(Vec::new());
        info.write_le(&mut buffer).unwrap();
        let bytes = buffer.into_inner();
        assert_eq!(
            &bytes[..20],
            &[
                0xc0, 0, 0, 0, 0xe2, 0x2e, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0x30, 0x75, 0, 0
            ]
        );
    }
}
