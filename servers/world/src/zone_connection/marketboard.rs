//! The marketboard's requests: searching a category, opening an item (its listings and history), and buying a
//! listing. What's listed and at what price is `crate::marketboard`; the packets are as retail sends them (Bob's
//! Chronofoil captures of 2026-09-15, see docs/src/fork/marketboard.md.)

use kawari::{
    common::ContainerType,
    ipc::zone::{
        ItemInfo, MARKETBOARD_HISTORY_ENTRIES, MARKETBOARD_ITEMS_PER_PAGE,
        MARKETBOARD_LISTINGS_PER_PAGE, MarketBoardHistory, MarketBoardItem, MarketBoardListing,
        MarketBoardPurchase, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};

use crate::{
    ItemInfoQuery, ZoneConnection,
    inventory::{CurrencyKind, CurrencyStorage, Item},
    marketboard::{self, MarketItem},
};

/// An item's HQ flag (`Item::item_flags`; ItemFlags.HighQuality in FFXIVClientStructs.)
const ITEM_FLAG_HQ: u8 = 1;

impl ZoneConnection {
    fn marketboard_item(&mut self, item_id: u32) -> Option<MarketItem> {
        let index = self.game_data.lock().marketboard();
        index.items.get(&item_id).cloned()
    }

    /// The item's listings right now, cheapest first.
    fn marketboard_listings_of(&mut self, item: &MarketItem) -> Vec<MarketBoardListing> {
        let mut database = self.database.lock();
        let prices = database.marketboard_prices(item);
        let settings = database.marketboard_settings();
        let sold = database.marketboard_sold(item.id);
        marketboard::listings(item, &prices, &settings, &sold)
    }

    /// A page of a search: the items of an ItemSearchCategory, with how many listings each has.
    pub async fn marketboard_search(
        &mut self,
        start_index: u32,
        request_id: u8,
        kind: u8,
        category: u16,
    ) {
        let index = self.game_data.lock().marketboard();
        let settings = self.database.lock().marketboard_settings();
        let ids = index
            .by_category
            .get(&category)
            .cloned()
            .unwrap_or_default();
        let start = (start_index as usize).min(ids.len());
        let end = (start + MARKETBOARD_ITEMS_PER_PAGE).min(ids.len());
        let items = ids[start..end]
            .iter()
            .map(|id| {
                let (nq, hq) = index
                    .items
                    .get(id)
                    .map(|item| marketboard::listing_counts(item, &settings))
                    .unwrap_or_default();
                MarketBoardItem {
                    item_id: *id,
                    count: nq + hq,
                }
            })
            .collect();
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::MarketBoardItems {
            items,
            next_index: if end < ids.len() { end as u32 } else { 0 },
            error: 0,
            start_index,
            request_id,
            kind,
        });
        self.send_ipc_self(ipc).await;
    }

    /// Opening an item: how many listings it has, then its history. The client then asks for the listings a page at
    /// a time (`marketboard_page`.)
    pub async fn marketboard_open(&mut self, item_id: u32) {
        self.marketboard_request_item_id = item_id;
        let Some(item) = self.marketboard_item(item_id) else {
            tracing::warn!("Marketboard: item {item_id} isn't on the marketboard");
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::MarketBoardHits { unk1: 0, hits: 0 });
            self.send_ipc_self(ipc).await;
            return;
        };
        let hits = self.marketboard_listings_of(&item).len() as u32;
        let entries = {
            let mut database = self.database.lock();
            let prices = database.marketboard_prices(&item);
            let settings = database.marketboard_settings();
            let sales = database.marketboard_sales(item_id, MARKETBOARD_HISTORY_ENTRIES);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|time| time.as_secs())
                .unwrap_or_default();
            marketboard::history(
                &item,
                &prices,
                &settings,
                &sales,
                now,
                MARKETBOARD_HISTORY_ENTRIES,
            )
        };

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::MarketBoardHits { unk1: 0, hits });
        self.send_ipc_self(ipc).await;
        let ipc =
            ServerZoneIpcSegment::new(ServerZoneIpcData::MarketBoardHistory(MarketBoardHistory {
                item_id,
                entries,
            }));
        self.send_ipc_self(ipc).await;
    }

    /// A page of the open item's listings.
    pub async fn marketboard_page(&mut self, start_index: u8, request_id: u8) {
        let item_id = self.marketboard_request_item_id;
        let listings = match self.marketboard_item(item_id) {
            Some(item) => self.marketboard_listings_of(&item),
            None => Vec::new(),
        };
        let start = (start_index as usize).min(listings.len());
        let end = (start + MARKETBOARD_LISTINGS_PER_PAGE).min(listings.len());
        // The index is a byte, so at most 250 or so listings can be paged through (retail shows 100.)
        let next_index = if end < listings.len() && end <= u8::MAX as usize {
            end as u8
        } else {
            0
        };
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::MarketBoardOfferings {
            listings: listings[start..end].to_vec(),
            next_index,
            start_index,
            request_id,
        });
        self.send_ipc_self(ipc).await;
    }

    /// Buying a listing: it has to still be there at the price the client saw, and the player needs the gil (price
    /// and tax) and room for it. The gil goes, the item arrives (HQ if the listing was), and the sale goes in the
    /// item's history; a new listing takes its place.
    #[allow(clippy::too_many_arguments)]
    pub async fn marketboard_purchase(
        &mut self,
        listing_id: u64,
        item_id: u32,
        quantity: u32,
        price_per_unit: u32,
    ) {
        let Some(item) = self.marketboard_item(item_id) else {
            self.send_notice("That item isn't on the marketboard.")
                .await;
            return;
        };
        let listings = self.marketboard_listings_of(&item);
        let Some(listing) = listings
            .iter()
            .find(|listing| listing.listing_id == listing_id)
            .cloned()
        else {
            self.send_notice("That listing is gone. Please check the listings again.")
                .await;
            return;
        };
        if listing.quantity != quantity || listing.price_per_unit != price_per_unit {
            self.send_notice("That listing's price has changed. Please check the listings again.")
                .await;
            return;
        }
        let total = (listing.price_per_unit as u64 * listing.quantity as u64
            + listing.total_tax as u64)
            .min(i32::MAX as u64) as i32;
        if self.player_data.inventory.currency.gil.quantity < total {
            self.send_notice("You don't have enough gil.").await;
            return;
        }
        let Some(row) = self
            .game_data
            .lock()
            .get_item_info(ItemInfoQuery::ById(item_id))
        else {
            self.send_notice("That item couldn't be found in the game data.")
                .await;
            return;
        };
        let mut bought = Item::new(&row, listing.quantity as i32);
        if listing.is_hq != 0 {
            bought.item_flags |= ITEM_FLAG_HQ;
        }
        // Room first (on a copy), then the sale (so the same listing can't be bought twice), then the item.
        if self
            .player_data
            .inventory
            .clone()
            .add_in_next_free_slot(bought)
            .is_none()
        {
            self.send_notice("Your inventory is full.").await;
            return;
        }
        let recorded = self.database.lock().record_marketboard_sale(
            item_id,
            listing_id,
            listing.is_hq != 0,
            listing.quantity,
            listing.price_per_unit,
            listing.total_tax,
            self.player_data.character.content_id as u64,
            &self.player_data.character.name,
        );
        if !recorded {
            self.send_notice("That listing is gone. Please check the listings again.")
                .await;
            return;
        }
        let Some(slot) = self.player_data.inventory.add_in_next_free_slot(bought) else {
            return;
        };
        self.player_data.inventory.currency.gil.quantity -= total;
        tracing::info!(
            "Marketboard: {} bought {} of item {item_id} (HQ: {}) for {total} gil",
            self.player_data.character.name,
            listing.quantity,
            listing.is_hq
        );

        // As retail: the gil, the item's slot, then the purchase.
        let gil = ItemInfo {
            sequence: self.player_data.item_sequence,
            container: ContainerType::Currency,
            slot: CurrencyStorage::get_slot_for_id(CurrencyKind::Gil),
            ..self.player_data.inventory.currency.gil.into()
        };
        self.send_ipc_self(ServerZoneIpcSegment::new(
            ServerZoneIpcData::UpdateInventorySlot(gil),
        ))
        .await;
        let slot = ItemInfo {
            sequence: self.player_data.item_sequence,
            ..slot
        };
        self.send_ipc_self(ServerZoneIpcSegment::new(
            ServerZoneIpcData::UpdateInventorySlot(slot),
        ))
        .await;
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::MarketBoardPurchase(
            MarketBoardPurchase {
                item_id,
                quantity: listing.quantity,
            },
        ));
        self.send_ipc_self(ipc).await;
    }
}
