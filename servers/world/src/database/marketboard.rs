//! The marketboard's tables (see `crate::marketboard` and docs/src/fork/marketboard.md.) Kawari Web writes the rates,
//! item prices and settings; they're read whenever the board is used, so a change applies at once.

use std::collections::HashSet;

use diesel::prelude::*;

use super::{WorldDatabase, schema, unixepoch};
use crate::marketboard::{
    LevelSource, MarketItem, Prices, RateChanges, Rates, Sale, Settings, default_rates,
};

type RateRow = (
    String,
    Option<i32>,
    Option<String>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
);

fn to_u32(value: Option<i32>) -> Option<u32> {
    value.map(|value| value.max(0) as u32)
}

fn changes(row: RateRow) -> RateChanges {
    let (_, per_level, level_source, white, pink, green, blue, purple, hq_multiplier) = row;
    RateChanges {
        per_level: to_u32(per_level),
        level_source: level_source.as_deref().and_then(LevelSource::parse),
        rarity: [white, pink, green, blue, purple].map(to_u32),
        hq_multiplier: to_u32(hq_multiplier),
    }
}

impl WorldDatabase {
    fn marketboard_rate_changes(&mut self, wanted: &str) -> RateChanges {
        use schema::marketboard_rate::dsl::*;

        marketboard_rate
            .filter(scope.eq(wanted))
            .first::<RateRow>(&mut self.connection)
            .map(changes)
            .unwrap_or_default()
    }

    /// The rates for an item: its tab's (the built-in ones where the table has nothing), with its search category's
    /// changes.
    pub fn marketboard_rates(&mut self, tab: u8, category: u16) -> Rates {
        let tab_rates =
            default_rates(tab).with(&self.marketboard_rate_changes(&format!("tab:{tab}")));
        tab_rates.with(&self.marketboard_rate_changes(&format!("category:{category}")))
    }

    /// An item's prices, from the rates unless it has its own.
    pub fn marketboard_prices(&mut self, item: &MarketItem) -> Prices {
        let rates = self.marketboard_rates(item.tab, item.category);
        let own = {
            use schema::marketboard_item::dsl::*;
            marketboard_item
                .filter(item_id.eq(item.id as i32))
                .select((nq_price, hq_price))
                .first::<(Option<i32>, Option<i32>)>(&mut self.connection)
                .ok()
        };
        let (nq, hq) = own.unwrap_or_default();
        Prices::of(item, &rates, to_u32(nq), to_u32(hq))
    }

    pub fn marketboard_settings(&mut self) -> Settings {
        use schema::marketboard_setting::dsl::*;

        let mut settings = Settings::default();
        let rows = marketboard_setting
            .load::<(String, i32)>(&mut self.connection)
            .unwrap_or_default();
        for (name, number) in rows {
            let number = number.max(0) as u32;
            match name.as_str() {
                "listings_per_item" => settings.listings_per_item = number,
                "hq_listings" => settings.hq_listings = number,
                "max_stack" => settings.max_stack = number.max(1),
                "tax_percent" => settings.tax_percent = number,
                _ => {}
            }
        }
        settings
    }

    /// The item's listings that have been bought.
    pub fn marketboard_sold(&mut self, item: u32) -> HashSet<u64> {
        use schema::marketboard_sale::dsl::*;

        marketboard_sale
            .filter(item_id.eq(item as i32))
            .select(listing_id)
            .load::<i64>(&mut self.connection)
            .unwrap_or_default()
            .into_iter()
            .map(|listing| listing as u64)
            .collect()
    }

    /// The item's latest sales, newest first.
    pub fn marketboard_sales(&mut self, item: u32, count: usize) -> Vec<Sale> {
        use schema::marketboard_sale::dsl::*;

        marketboard_sale
            .filter(item_id.eq(item as i32))
            .order(id.desc())
            .limit(count as i64)
            .select((
                listing_id,
                hq,
                quantity,
                price_per_unit,
                buyer_name,
                sold_at,
            ))
            .load::<(i64, i32, i32, i32, String, i64)>(&mut self.connection)
            .unwrap_or_default()
            .into_iter()
            .map(|(listing, is_hq, amount, price, name, when)| Sale {
                listing_id: listing as u64,
                hq: is_hq != 0,
                quantity: amount.max(0) as u32,
                price_per_unit: price.max(0) as u32,
                buyer_name: name,
                sold_at: when.max(0) as u64,
            })
            .collect()
    }

    /// Records a purchase. Returns false if that listing had already been bought.
    #[allow(clippy::too_many_arguments)]
    pub fn record_marketboard_sale(
        &mut self,
        item: u32,
        listing: u64,
        is_hq: bool,
        amount: u32,
        price: u32,
        paid_tax: u32,
        buyer: u64,
        name: &str,
    ) -> bool {
        use schema::marketboard_sale::dsl::*;

        if self.marketboard_sold(item).contains(&listing) {
            return false;
        }
        let now = diesel::select(unixepoch())
            .get_result::<i64>(&mut self.connection)
            .unwrap_or_default();
        diesel::insert_into(marketboard_sale)
            .values((
                item_id.eq(item as i32),
                listing_id.eq(listing as i64),
                hq.eq(is_hq as i32),
                quantity.eq(amount as i32),
                price_per_unit.eq(price.min(i32::MAX as u32) as i32),
                tax.eq(paid_tax.min(i32::MAX as u32) as i32),
                buyer_content_id.eq(buyer as i64),
                buyer_name.eq(name),
                sold_at.eq(now),
            ))
            .execute(&mut self.connection)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use diesel::{Connection, SqliteConnection};
    use diesel_migrations::MigrationHarness;

    use super::*;
    use crate::database::MIGRATIONS;
    use crate::marketboard::{TAB_ITEMS, TAB_WEAPONS};

    fn database() -> WorldDatabase {
        let mut connection = SqliteConnection::establish(":memory:").unwrap();
        connection.run_pending_migrations(MIGRATIONS).unwrap();
        WorldDatabase { connection }
    }

    fn sword() -> MarketItem {
        MarketItem {
            id: 1601,
            category: 10,
            tab: TAB_WEAPONS,
            rarity: 2,
            level_equip: 50,
            level_item: 130,
            can_be_hq: true,
            stack_size: 1,
        }
    }

    #[test]
    fn seeded_rates_are_the_defaults() {
        let mut db = database();
        for tab in 1..=4 {
            assert_eq!(db.marketboard_rates(tab, 0), default_rates(tab));
        }
        assert_eq!(db.marketboard_settings(), Settings::default());
    }

    #[test]
    fn changes_override_in_order() {
        let mut db = database();
        diesel::sql_query("UPDATE marketboard_rate SET per_level = 200 WHERE scope = 'tab:1'")
            .execute(&mut db.connection)
            .unwrap();
        diesel::sql_query(
            "INSERT INTO marketboard_rate (scope, rarity_green) VALUES ('category:10', 1000)",
        )
        .execute(&mut db.connection)
        .unwrap();
        // 200 × 50 + 1,000; HQ 200 × 50 + 2,000.
        assert_eq!(
            db.marketboard_prices(&sword()),
            Prices {
                nq: 11000,
                hq: 12000
            }
        );
        diesel::sql_query("INSERT INTO marketboard_item (item_id, hq_price) VALUES (1601, 77)")
            .execute(&mut db.connection)
            .unwrap();
        assert_eq!(
            db.marketboard_prices(&sword()),
            Prices { nq: 11000, hq: 77 }
        );
        // Another tab isn't touched.
        assert_eq!(
            db.marketboard_rates(TAB_ITEMS, 49),
            default_rates(TAB_ITEMS)
        );
    }

    #[test]
    fn a_listing_sells_once() {
        let mut db = database();
        assert!(db.record_marketboard_sale(1601, 42, true, 1, 13000, 650, 7, "Bob"));
        assert!(!db.record_marketboard_sale(1601, 42, true, 1, 13000, 650, 7, "Bob"));
        assert_eq!(db.marketboard_sold(1601), HashSet::from([42]));
        let sales = db.marketboard_sales(1601, 20);
        assert_eq!(
            (sales.len(), sales[0].buyer_name.as_str(), sales[0].hq),
            (1, "Bob", true)
        );
    }
}
