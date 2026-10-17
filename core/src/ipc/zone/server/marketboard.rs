use binrw::binrw;

use crate::common::{CHAR_NAME_MAX_LENGTH, read_string, write_string};

// The layouts below were checked byte for byte against retail (Bob's Chronofoil captures of 2026-09-15: 44 searches,
// 40 items opened, 62 pages of listings, two purchases), see docs/src/fork/marketboard.md.

/// One item in a marketboard search result.
// TODO: what about favorites and wishlists?
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct MarketBoardItem {
    /// Index into the Item Excel sheet.
    pub item_id: u32,
    /// How many offers of this item are available. Retail sometimes sets the upper 16 bits (1, 2 or 5), meaning
    /// unknown.
    pub count: u32,
}

impl MarketBoardItem {
    pub const SIZE: usize = 8;
}

/// How many items a search result page holds; the client asks for the next page at `next_index`.
pub const MARKETBOARD_ITEMS_PER_PAGE: usize = 20;

#[binrw]
#[derive(Debug, Clone, Default)]
pub struct MarketBoardHistoryEntry {
    /// In Gil, per item.
    pub price: u32,
    /// When it sold (Unix seconds.)
    pub timestamp: u32,
    /// How many items were sold.
    pub quantity: u32,
    pub is_hq: u8,
    pub on_mannequin: u8,
    /// The buyer.
    #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
    #[br(count = CHAR_NAME_MAX_LENGTH)]
    #[br(map = read_string)]
    #[bw(map = write_string)]
    #[brw(pad_after = 2)] // padding
    pub name: String,
}

impl MarketBoardHistoryEntry {
    pub const SIZE: usize = 48;
}

/// How many sales an item's history holds.
pub const MARKETBOARD_HISTORY_ENTRIES: usize = 20;

#[binrw]
#[derive(Debug, Clone, Default)]
pub struct MarketBoardHistory {
    /// Index into the Item Excel sheet.
    pub item_id: u32,
    #[br(count = MARKETBOARD_HISTORY_ENTRIES)]
    #[brw(pad_size_to = MarketBoardHistoryEntry::SIZE * MARKETBOARD_HISTORY_ENTRIES)]
    #[brw(pad_after = 4)] // padding
    pub entries: Vec<MarketBoardHistoryEntry>,
}

impl MarketBoardHistory {
    pub const SIZE: usize = 968;
}

/// One listing of an item, as the marketboard shows it.
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct MarketBoardListing {
    /// Unique to the listing; the client sends it back to buy it.
    pub listing_id: u64,
    pub retainer_id: u64,
    pub retainer_owner_id: u64,
    /// The crafter's content ID, 0 if it wasn't crafted (or is unsigned.)
    pub artisan_id: u64,
    /// In Gil.
    pub price_per_unit: u32,
    /// The tax the buyer pays on top of `price_per_unit * quantity` (5% on retail.)
    pub total_tax: u32,
    pub quantity: u32,
    /// Index into the Item Excel sheet (without the HQ offset; see `is_hq`.)
    pub item_id: u32,
    /// Retail sends small numbers (1 to 14), probably the minutes since the retainer last updated it.
    pub last_review_time: u16,
    /// The retainer's market container: 0x7530 (30000) on retail.
    pub container_id: u16,
    /// Unknown, 0 on retail.
    pub unk1: u16,
    /// The melded materia, if any.
    pub materia: [u16; 5],
    /// Unknown, always 0 on retail.
    pub unk2: [u8; 6],
    #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
    #[br(count = CHAR_NAME_MAX_LENGTH)]
    #[br(map = read_string)]
    #[bw(map = write_string)]
    pub retainer_name: String,
    /// Always 0 on retail (once the crafter's name?)
    pub unk3: [u8; 32],
    pub is_hq: u8,
    pub materia_count: u8,
    pub on_mannequin: u8,
    /// Where the retainer is: 1 Limsa Lominsa, 2 Gridania, 3 Ul'dah, 4 Ishgard, 7 Kugane, 10 the Crystarium, 12 Old
    /// Sharlayan, 14 Tuliyollal (as retail shows them.)
    pub retainer_city: u8,
    #[brw(pad_after = 4)] // padding
    pub stain: u16,
}

impl MarketBoardListing {
    pub const SIZE: usize = 144;
}

/// How many listings a page of an item's offerings holds.
pub const MARKETBOARD_LISTINGS_PER_PAGE: usize = 10;

/// The server's answer to a purchase (`MarketBoardPurchaseHandler`): what was bought.
#[binrw]
#[derive(Debug, Clone, Default)]
pub struct MarketBoardPurchase {
    /// Index into the Item Excel sheet.
    #[brw(pad_after = 4)] // empty
    pub item_id: u32,
    #[brw(pad_after = 4)] // empty
    pub quantity: u32,
}

impl MarketBoardPurchase {
    pub const SIZE: usize = 16;
}

#[cfg(test)]
mod tests {
    use crate::common::ensure_size;

    use super::*;

    #[test]
    fn marketboard_item_size() {
        ensure_size::<MarketBoardItem, { MarketBoardItem::SIZE }>();
    }

    #[test]
    fn marketboard_history_size() {
        ensure_size::<MarketBoardHistoryEntry, { MarketBoardHistoryEntry::SIZE }>();
    }

    #[test]
    fn marketboard_listing_size() {
        ensure_size::<MarketBoardListing, { MarketBoardListing::SIZE }>();
    }

    #[test]
    fn marketboard_purchase_size() {
        ensure_size::<MarketBoardPurchase, { MarketBoardPurchase::SIZE }>();
    }
}
