//! The marketboard: which items are on it, what they cost, and the listings and history it shows.
//!
//! Every item with an ItemSearchCategory is listed. Its price comes from rates an administrator sets in Kawari Web
//! (the `marketboard_*` tables, see `database/marketboard.rs`): per tab (Weapons, Armor, Items, Housing), changed per
//! search category, or fixed per item. Listings are made up from the item and the settings, the same every time
//! (seeded by the item), except that a listing someone bought is replaced by the next one. See
//! docs/src/fork/marketboard.md.

use std::collections::HashSet;

use kawari::ipc::zone::{MarketBoardHistoryEntry, MarketBoardListing};

/// The marketboard's tabs, ItemSearchCategory's Category.
pub const TAB_WEAPONS: u8 = 1;
pub const TAB_ARMOR: u8 = 2;
pub const TAB_ITEMS: u8 = 3;
pub const TAB_HOUSING: u8 = 4;

/// The container retail's listings say they're in (a retainer's market.)
const LISTING_CONTAINER: u16 = 30000;
/// The cities retail's retainers are in, as the listing's `retainer_city` (see `MarketBoardListing`.)
const RETAINER_CITIES: [u8; 8] = [1, 2, 3, 4, 7, 10, 12, 14];
/// How long ago the made-up sales in an item's history are, at most.
const HISTORY_SPAN_SECONDS: u64 = 7 * 24 * 60 * 60;

/// What an item's level is, for its price.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelSource {
    /// The level to equip it (LevelEquip): what a weapon or armor is "for".
    Equip,
    /// Its item level (LevelItem): the only level materials and furnishings have.
    Item,
}

impl LevelSource {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "equip" => Some(Self::Equip),
            "item" => Some(Self::Item),
            _ => None,
        }
    }
}

/// Rarity, as the bonus index: white, pink, green, blue, purple.
pub fn rarity_index(rarity: u8) -> usize {
    match rarity {
        7 => 1, // pink
        2 => 2, // green
        3 => 3, // blue
        4 => 4, // purple
        _ => 0, // white (1), and anything else
    }
}

/// A tab's rates: price = per_level × level + the rarity's bonus, the bonus times `hq_multiplier` when HQ.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rates {
    pub per_level: u32,
    pub level_source: LevelSource,
    /// White, pink, green, blue, purple.
    pub rarity: [u32; 5],
    pub hq_multiplier: u32,
}

/// The rates a tab has until an administrator changes them (Bob's, 2026-10-17): weapons and armor 100 gil a level
/// with 0/2,000/4,000/8,000/20,000 by rarity; items (crafting supplies and the rest) 3 a level with those bonuses
/// divided by 100; housing 50 a level with the full bonuses; HQ doubles the bonus.
pub fn default_rates(tab: u8) -> Rates {
    const EQUIPMENT: [u32; 5] = [0, 2000, 4000, 8000, 20000];
    match tab {
        TAB_WEAPONS | TAB_ARMOR => Rates {
            per_level: 100,
            level_source: LevelSource::Equip,
            rarity: EQUIPMENT,
            hq_multiplier: 2,
        },
        TAB_HOUSING => Rates {
            per_level: 50,
            level_source: LevelSource::Item,
            rarity: EQUIPMENT,
            hq_multiplier: 2,
        },
        _ => Rates {
            per_level: 3,
            level_source: LevelSource::Item,
            rarity: EQUIPMENT.map(|bonus| bonus / 100),
            hq_multiplier: 2,
        },
    }
}

/// Some of a tab's rates, changed (a search category's); `None` keeps the tab's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RateChanges {
    pub per_level: Option<u32>,
    pub level_source: Option<LevelSource>,
    pub rarity: [Option<u32>; 5],
    pub hq_multiplier: Option<u32>,
}

impl Rates {
    pub fn with(&self, changes: &RateChanges) -> Rates {
        let mut rarity = self.rarity;
        for (bonus, change) in rarity.iter_mut().zip(changes.rarity) {
            if let Some(change) = change {
                *bonus = change;
            }
        }
        Rates {
            per_level: changes.per_level.unwrap_or(self.per_level),
            level_source: changes.level_source.unwrap_or(self.level_source),
            rarity,
            hq_multiplier: changes.hq_multiplier.unwrap_or(self.hq_multiplier),
        }
    }
}

/// The marketboard's settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Listings each item always has.
    pub listings_per_item: u32,
    /// How many of them are HQ, for items that can be.
    pub hq_listings: u32,
    /// The largest stack a listing has (or the item's stack size, if smaller.)
    pub max_stack: u32,
    /// The tax buyers pay on top, in percent (5 on retail.)
    pub tax_percent: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            listings_per_item: 50,
            hq_listings: 10,
            max_stack: 99,
            tax_percent: 5,
        }
    }
}

/// An item that's on the marketboard, with what its price and listings need.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MarketItem {
    pub id: u32,
    /// ItemSearchCategory.
    pub category: u16,
    /// The marketboard tab the category is on (see `TAB_*`.)
    pub tab: u8,
    pub rarity: u8,
    pub level_equip: u8,
    pub level_item: u16,
    pub can_be_hq: bool,
    pub stack_size: u32,
}

/// The price of one of the item, in gil (at least 1.)
pub fn price(item: &MarketItem, rates: &Rates, hq: bool) -> u32 {
    let level = match rates.level_source {
        LevelSource::Equip => item.level_equip as u32,
        LevelSource::Item => item.level_item as u32,
    };
    let mut bonus = rates.rarity[rarity_index(item.rarity)];
    if hq {
        bonus = bonus.saturating_mul(rates.hq_multiplier);
    }
    rates
        .per_level
        .saturating_mul(level)
        .saturating_add(bonus)
        .max(1)
}

/// The item's prices: NQ and HQ, from the rates unless the item has its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prices {
    pub nq: u32,
    pub hq: u32,
}

impl Prices {
    pub fn of(
        item: &MarketItem,
        rates: &Rates,
        nq_override: Option<u32>,
        hq_override: Option<u32>,
    ) -> Self {
        Self {
            nq: nq_override
                .unwrap_or_else(|| price(item, rates, false))
                .max(1),
            hq: hq_override
                .unwrap_or_else(|| price(item, rates, true))
                .max(1),
        }
    }

    pub fn get(&self, hq: bool) -> u32 {
        if hq { self.hq } else { self.nq }
    }
}

/// The tax on a purchase, as retail rounds it (down: 44,998 gil at 5% is 2,249.)
pub fn tax(price_per_unit: u32, quantity: u32, tax_percent: u32) -> u32 {
    (price_per_unit as u64 * quantity as u64 * tax_percent as u64 / 100).min(u32::MAX as u64) as u32
}

/// A small, fixed random number generator (SplitMix64), so an item's listings come out the same every time.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// In 0..n.
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }
}

/// A listing's ID: the item, whether it's HQ, and its place in the item's list of listings. The top bit keeps them
/// apart from anything else.
pub fn listing_id(item_id: u32, hq: bool, serial: u32) -> u64 {
    (1u64 << 63) | ((item_id as u64) << 24) | ((hq as u64) << 23) | (serial as u64 & 0x7F_FFFF)
}

/// Made-up names for the retainers and buyers, from syllables (not from the game's text.)
const SYLLABLES: [&str; 32] = [
    "ka", "ri", "no", "ta", "mi", "ra", "lo", "ve", "su", "en", "da", "ko", "li", "ma", "ro", "sa",
    "fi", "ne", "tu", "ba", "el", "yu", "go", "ha", "ze", "an", "wi", "pe", "ol", "chi", "dra",
    "mo",
];

fn made_up_word(rng: &mut Rng, syllables: u64) -> String {
    let mut word = String::new();
    for _ in 0..syllables {
        word.push_str(SYLLABLES[rng.below(SYLLABLES.len() as u64) as usize]);
    }
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => word,
    }
}

/// A retainer's name (one word, as retail's are.)
fn retainer_name(rng: &mut Rng) -> String {
    let syllables = 2 + rng.below(2);
    made_up_word(rng, syllables)
}

/// A character's name (two words.)
fn character_name(rng: &mut Rng) -> String {
    let first = 2 + rng.below(2);
    let last = 2 + rng.below(2);
    format!("{} {}", made_up_word(rng, first), made_up_word(rng, last))
}

/// How many of each quality the item lists.
pub fn listing_counts(item: &MarketItem, settings: &Settings) -> (u32, u32) {
    let hq = if item.can_be_hq {
        settings.hq_listings.min(settings.listings_per_item)
    } else {
        0
    };
    (settings.listings_per_item - hq, hq)
}

/// One listing of the item, the `serial`th of its quality.
fn listing(
    item: &MarketItem,
    prices: &Prices,
    settings: &Settings,
    hq: bool,
    serial: u32,
) -> MarketBoardListing {
    let id = listing_id(item.id, hq, serial);
    let mut rng = Rng::new(id);
    let largest = item.stack_size.min(settings.max_stack).max(1);
    let quantity = 1 + rng.below(largest as u64) as u32;
    let price_per_unit = prices.get(hq);
    let retainer_id = rng.next() & 0x00FF_FFFF_FFFF_FFFF;
    MarketBoardListing {
        listing_id: id,
        retainer_id,
        retainer_owner_id: 0,
        artisan_id: 0,
        price_per_unit,
        total_tax: tax(price_per_unit, quantity, settings.tax_percent),
        quantity,
        item_id: item.id,
        last_review_time: 1 + rng.below(14) as u16,
        container_id: LISTING_CONTAINER,
        retainer_name: retainer_name(&mut rng),
        is_hq: hq as u8,
        retainer_city: RETAINER_CITIES[rng.below(RETAINER_CITIES.len() as u64) as usize],
        ..Default::default()
    }
}

/// The item's listings, cheapest first: `listing_counts` of each quality, skipping the ones in `sold`.
pub fn listings(
    item: &MarketItem,
    prices: &Prices,
    settings: &Settings,
    sold: &HashSet<u64>,
) -> Vec<MarketBoardListing> {
    let (nq_count, hq_count) = listing_counts(item, settings);
    let mut all = Vec::with_capacity((nq_count + hq_count) as usize);
    for (hq, count) in [(false, nq_count), (true, hq_count)] {
        let mut serial = 0;
        let mut made = 0;
        while made < count && serial < 0x7F_FFFF {
            if !sold.contains(&listing_id(item.id, hq, serial)) {
                all.push(listing(item, prices, settings, hq, serial));
                made += 1;
            }
            serial += 1;
        }
    }
    all.sort_by_key(|listing| (listing.price_per_unit, listing.is_hq, listing.listing_id));
    all
}

/// A sale, from the database.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sale {
    pub listing_id: u64,
    pub hq: bool,
    pub quantity: u32,
    pub price_per_unit: u32,
    pub buyer_name: String,
    pub sold_at: u64,
}

/// The item's history (`count` sales, newest first): the real ones, then made-up ones at its current prices over the
/// last week so the tab isn't empty. The made-up ones only change from one day to the next.
pub fn history(
    item: &MarketItem,
    prices: &Prices,
    settings: &Settings,
    sales: &[Sale],
    now: u64,
    count: usize,
) -> Vec<MarketBoardHistoryEntry> {
    let mut entries: Vec<MarketBoardHistoryEntry> = sales
        .iter()
        .take(count)
        .map(|sale| MarketBoardHistoryEntry {
            price: sale.price_per_unit,
            timestamp: sale.sold_at as u32,
            quantity: sale.quantity,
            is_hq: sale.hq as u8,
            on_mannequin: 0,
            name: sale.buyer_name.clone(),
        })
        .collect();

    let day = now / (24 * 60 * 60);
    let mut rng = Rng::new(((item.id as u64) << 32) ^ day);
    let largest = item.stack_size.min(settings.max_stack).max(1);
    let (_, hq_count) = listing_counts(item, settings);
    let oldest = entries
        .last()
        .map(|entry| entry.timestamp as u64)
        .unwrap_or(now);
    let mut made_up = Vec::new();
    while entries.len() + made_up.len() < count {
        let hq =
            hq_count > 0 && rng.below(settings.listings_per_item.max(1) as u64) < hq_count as u64;
        let ago = rng.below(HISTORY_SPAN_SECONDS);
        made_up.push(MarketBoardHistoryEntry {
            price: prices.get(hq),
            timestamp: oldest.saturating_sub(ago) as u32,
            quantity: 1 + rng.below(largest as u64) as u32,
            is_hq: hq as u8,
            on_mannequin: 0,
            name: character_name(&mut rng),
        });
    }
    made_up.sort_by_key(|entry| std::cmp::Reverse(entry.timestamp));
    entries.extend(made_up);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sword() -> MarketItem {
        MarketItem {
            id: 1601,
            category: 10,
            tab: TAB_WEAPONS,
            rarity: 2, // green
            level_equip: 50,
            level_item: 130,
            can_be_hq: true,
            stack_size: 1,
        }
    }

    fn lumber() -> MarketItem {
        MarketItem {
            id: 5380,
            category: 49,
            tab: TAB_ITEMS,
            rarity: 1,
            level_equip: 1,
            level_item: 20,
            can_be_hq: true,
            stack_size: 999,
        }
    }

    #[test]
    fn bobs_prices() {
        let weapons = default_rates(TAB_WEAPONS);
        // 100 a level + green's 4,000; HQ doubles the bonus.
        assert_eq!(price(&sword(), &weapons, false), 9000);
        assert_eq!(price(&sword(), &weapons, true), 13000);
        let purple = MarketItem {
            rarity: 4,
            ..sword()
        };
        assert_eq!(price(&purple, &weapons, false), 25000);
        assert_eq!(price(&purple, &weapons, true), 45000);
        let pink = MarketItem {
            rarity: 7,
            ..sword()
        };
        assert_eq!(price(&pink, &weapons, false), 7000);
        // Items: 3 an item level, bonuses divided by 100.
        let items = default_rates(TAB_ITEMS);
        assert_eq!(price(&lumber(), &items, false), 60);
        let blue = MarketItem {
            rarity: 3,
            ..lumber()
        };
        assert_eq!(price(&blue, &items, false), 140);
        assert_eq!(price(&blue, &items, true), 220);
        // Housing: 50 an item level, the full bonuses.
        let table = MarketItem {
            tab: TAB_HOUSING,
            rarity: 2,
            level_item: 10,
            ..lumber()
        };
        assert_eq!(price(&table, &default_rates(TAB_HOUSING), false), 4500);
        // White HQ: the bonus is 0, so the same as NQ.
        assert_eq!(price(&lumber(), &items, true), 60);
    }

    #[test]
    fn category_changes_and_item_prices() {
        let changes = RateChanges {
            per_level: Some(10),
            rarity: [None, None, Some(5), None, None],
            ..Default::default()
        };
        let rates = default_rates(TAB_ITEMS).with(&changes);
        assert_eq!(rates.per_level, 10);
        assert_eq!(rates.rarity, [0, 20, 5, 80, 200]);
        assert_eq!(rates.level_source, LevelSource::Item);
        let prices = Prices::of(&sword(), &default_rates(TAB_WEAPONS), Some(123), None);
        assert_eq!(prices, Prices { nq: 123, hq: 13000 });
    }

    #[test]
    fn fifty_listings_ten_hq() {
        let settings = Settings::default();
        let item = lumber();
        let prices = Prices::of(&item, &default_rates(TAB_ITEMS), None, None);
        let listings = listings(&item, &prices, &settings, &HashSet::new());
        assert_eq!(listings.len(), 50);
        assert_eq!(listings.iter().filter(|l| l.is_hq == 1).count(), 10);
        assert!(listings.iter().all(|l| (1..=99).contains(&l.quantity)));
        // Random stacks, not all the same.
        assert!(
            listings
                .iter()
                .map(|l| l.quantity)
                .collect::<HashSet<_>>()
                .len()
                > 10
        );
        assert!(
            listings
                .iter()
                .all(|l| l.total_tax == tax(l.price_per_unit, l.quantity, 5))
        );
        let ids: HashSet<u64> = listings.iter().map(|l| l.listing_id).collect();
        assert_eq!(ids.len(), 50);
        // The same every time.
        let again = super::listings(&item, &prices, &settings, &HashSet::new());
        assert_eq!(
            again
                .iter()
                .map(|l| (l.listing_id, l.quantity))
                .collect::<Vec<_>>(),
            listings
                .iter()
                .map(|l| (l.listing_id, l.quantity))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn stacks_and_qualities() {
        let settings = Settings::default();
        // Unstackable: one each.
        let item = sword();
        let prices = Prices::of(&item, &default_rates(TAB_WEAPONS), None, None);
        assert!(
            listings(&item, &prices, &settings, &HashSet::new())
                .iter()
                .all(|l| l.quantity == 1)
        );
        // Stacks below 99: at most the stack size.
        let small = MarketItem {
            stack_size: 20,
            can_be_hq: false,
            ..lumber()
        };
        let listed = listings(&small, &prices, &settings, &HashSet::new());
        assert!(listed.iter().all(|l| (1..=20).contains(&l.quantity)));
        // Can't be HQ: all 50 NQ.
        assert_eq!(listed.len(), 50);
        assert!(listed.iter().all(|l| l.is_hq == 0));
    }

    #[test]
    fn sold_listings_are_replaced() {
        let settings = Settings::default();
        let item = lumber();
        let prices = Prices::of(&item, &default_rates(TAB_ITEMS), None, None);
        let before = listings(&item, &prices, &settings, &HashSet::new());
        let bought = before.iter().find(|l| l.is_hq == 1).unwrap().listing_id;
        let after = listings(&item, &prices, &settings, &HashSet::from([bought]));
        assert_eq!(after.len(), 50);
        assert_eq!(after.iter().filter(|l| l.is_hq == 1).count(), 10);
        assert!(after.iter().all(|l| l.listing_id != bought));
    }

    #[test]
    fn history_fills_up() {
        let settings = Settings::default();
        let item = lumber();
        let prices = Prices::of(&item, &default_rates(TAB_ITEMS), None, None);
        let now = 1_792_300_000;
        let sales = vec![Sale {
            listing_id: 1,
            hq: true,
            quantity: 7,
            price_per_unit: 60,
            buyer_name: "Bob".to_string(),
            sold_at: now - 10,
        }];
        let entries = history(&item, &prices, &settings, &sales, now, 20);
        assert_eq!(entries.len(), 20);
        assert_eq!((entries[0].quantity, entries[0].name.as_str()), (7, "Bob"));
        assert!(entries.windows(2).all(|w| w[0].timestamp >= w[1].timestamp));
        assert!(
            entries
                .iter()
                .all(|e| !e.name.is_empty() && e.name.len() <= 31)
        );
    }

    #[test]
    fn tax_rounds_down() {
        assert_eq!(tax(35000, 1, 5), 1750);
        assert_eq!(tax(44998, 1, 5), 2249);
    }
}
