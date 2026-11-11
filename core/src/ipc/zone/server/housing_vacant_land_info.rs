use binrw::binrw;

use crate::ipc::zone::{
    AvailabilityType, PlotLottoBidType, PlotLottoResult, PurchaseType, TenantType,
};

/// Represents a vacant housing plot.
#[binrw]
#[derive(Debug, Default, Clone)]
pub struct HousingVacantLandInfo {
    pub purchase_type: PurchaseType,
    pub tenant_type: TenantType,
    pub availability_type: AvailabilityType,
    /// The player's result during this bidding period.
    pub lottery_result: PlotLottoResult,
    /// The type of bid the player placed on this plot.
    #[brw(pad_after = 3)] // Presumably empty
    pub bid_type: PlotLottoBidType,
    /// A Unix timestamp indicating when this bidding period expires.
    pub phase_ends_at: u32,
    /// A Unix timestamp seemingly indicating when the player's refund or win claim expires.
    pub claim_ends_at: u32, // TODO: Is this used for both wins and refunds, or only refunds? For refunds it seems to be the bidding period expiration plus three months.
    /// How many bids were placed during this bidding period.
    pub entry_count: u32,
    /// The winning bid number.
    pub winning_bid_number: u32,
    /// The winning bid number, repeated (why?).
    pub winning_bid_number_again: u32,
    pub unk1: u32, // TODO: Has data, but not sure what it is yet
    /// Unknown. Seemingly always 0xFFFFFFFF.
    pub unk2: u32,
    /// Unknown. Seemingly always 0xFFFFFFFF.
    pub unk3: u32,
}
