# Marketboard

Every tradable item is on the marketboard, always with a full page of listings, at prices an administrator sets in
Kawari Web. Players can search the categories, open an item (its listings and history) and buy, which takes the gil
(and tax) and puts the item in their inventory. There are no players' retainers behind it: the listings are made up
by the server from the item and the settings.

* Server: `servers/world/src/marketboard.rs` (prices, listings, history), `zone_connection/marketboard.rs` (the
  requests), `database/marketboard.rs` (the tables), `GameData::marketboard` (which items, by category).
* Packets: `core/src/ipc/zone/server/marketboard.rs`, the `MarketBoard*` variants, two fork opcodes
  ([Fork Opcodes](opcodes.md)).
* Admin page: Kawari Web's **Dashboard → Marketboard** ([Kawari Web](kawari-web.md#marketboard-dashboard--marketboard)).
* The marketboard object's event: `events/custom/000/CmnDefMarketBoardGridania_00039.lua`.

## What's listed

Every item with an **ItemSearchCategory** (the Item sheet's column), which is how the game itself says what can be on
the board; untradable items have none and are skipped. The category's **Category** column is its tab: 1 Weapons,
2 Armor, 3 Items, 4 Housing. `GameData::marketboard` builds the list once, the first time anyone uses the board, from
the Item sheet already in memory. ItemSearchCategory has a Name column, so it only exists per language and is read in
the server's language (read with `Language::None` it fails to resolve, which left the first build's board empty). An
index whose categories couldn't be read isn't kept, so it's tried again. A search returns the category's items by item ID: the order retail sends them in
isn't the item level or the ID, and isn't known.

## Prices

```
price = per_level × level + rarity bonus          (at least 1 gil)
HQ price = per_level × level + rarity bonus × hq_multiplier
```

* **level** is the item's equip level (LevelEquip) or its item level (LevelItem), per the rates' `level_source`.
* **rarity bonus** by the Item sheet's Rarity: 1 white, 7 pink, 2 green, 3 blue, 4 purple.
* The tax buyers pay on top is `price × quantity × tax_percent / 100`, rounded down (retail: 35,000 gil → 1,750;
  44,998 → 2,249).

Built-in rates (Bob's, 2026-10-17; the migration seeds them, and the server falls back to them):

| Tab | Gil per level | Level | White | Pink | Green | Blue | Purple | HQ × bonus |
|---|---|---|---|---|---|---|---|---|
| 1 Weapons | 100 | equip | 0 | 2,000 | 4,000 | 8,000 | 20,000 | 2 |
| 2 Armor | 100 | equip | 0 | 2,000 | 4,000 | 8,000 | 20,000 | 2 |
| 3 Items (crafting supplies and everything else) | 3 | item | 0 | 20 | 40 | 80 | 200 | 2 |
| 4 Housing | 50 | item | 0 | 2,000 | 4,000 | 8,000 | 20,000 | 2 |

E.g. a level 50 green sword: 9,000 NQ, 13,000 HQ; item level 20 white lumber: 60 (and 60 HQ, since white's bonus is
0); a blue material at item level 20: 140 NQ, 220 HQ.

The settings stack the way the board is organised:

1. the **tab's** rates (`marketboard_rate`, scope `tab:N`);
2. a **search category's** changes (scope `category:N`), where each NULL keeps the tab's value;
3. an **item's own** NQ and/or HQ price (`marketboard_item`), where NULL keeps the computed one.

The server reads them whenever the board is used (a few small queries per item opened), so a change in Kawari Web
applies the next time anyone looks, with no restart. `marketboard::price` and Kawari Web's
`kawari_core/marketboard.py` `price` must stay the same formula; both are checked against the same examples
(`marketboard::tests::bobs_prices`).

## The order of a category's items

* **Retail's order**: by the item's ItemSortCategory's `Param`, then its `SubcategorySort`, then its ID. In Bob's
  Chronofoil captures of 2026-09-15 (8 categories browsed on the Items tab, 400-odd items) 7 categories came in exactly
  that order; the eighth (74) had two of its sort categories the other way round. Kawari listed them by item ID until
  2026-10-23 (`GameData::marketboard`; workspace icarus feature `ItemSortCategory`).

## The Weapons and Armor tabs' filters

On those tabs the window has a level box and a Class/Job dropdown, both set to the player's own when it opens, and the
search shows only what they allow. The client sends them in every page's request: `flags` 1, then the level, then
the ClassJob (0 for any). Bob's bundles: 2026-10-23, 18 and later 19 as his Arcanist levelled, class 0 on the weapon
categories and 6 once he was a Conjurer; 2026-10-26, a Conjurer 21 on category 37, 21 with the dropdown at 6, 37 and
24. (Until 2026-10-26 Kawari read the level as a sort order, 0x13 and 0x12, and listed every item of the category.)

Kawari keeps the category's items (still in retail's order) whose `LevelEquip` is at most the level and whose
`ClassJobCategory` row includes the ClassJob (`marketboard::search_filter`, `filtered`; the rows each ClassJob is in
come from the ClassJobCategory sheet's columns, read with the index, `GameData::classjob_category_rows`). Level 0 or
class 0 doesn't filter. The Items and Housing tabs send `flags` 0 and are never filtered. Whether retail keeps items
*at or below* the level, rather than exactly it or a range, isn't recorded: no retail capture has browsed these tabs.

The text after the two bytes ("Acolyte's" on 2026-10-26, "Beetle Glue" on the Items tab the same morning, carried
into categories it doesn't belong to) is the search box's, which the client doesn't clear between searches; Kawari
doesn't filter by it.

## Listings

`marketboard_setting`: `listings_per_item` (50), `hq_listings` (10), `max_stack` (99), `tax_percent` (5).

* An item lists `listings_per_item`, of which `hq_listings` HQ when the item can be HQ (Item.CanBeHq); otherwise
  all NQ.
* Each listing's quantity is random from 1 to `max_stack`, or the item's stack size if smaller (unstackable items:
  1). Retainer names are made up from syllables (not game text), and so are the retainer IDs, cities (retail's 1, 2,
  3, 4, 7, 10, 12, 14) and "last reviewed" minutes.
* They're the same every time: each comes from a fixed random generator seeded by its **listing ID**, which is the
  item, HQ or not, and its place in the item's list (`marketboard::listing_id`: top bit set, item ID, HQ bit, a
  23-bit serial).
* **A bought listing is replaced**: purchases are recorded in `marketboard_sale` with the listing ID, and the
  generator skips those and makes the next serial instead, so the item always has its full count.
* Sorted cheapest first (all of a quality share a price, so NQ before HQ unless an item's own prices say otherwise).

**History**: the item's real sales (newest first, from `marketboard_sale`), then made-up ones at its current prices
over the week before, so the tab isn't empty (they change from one day to the next, not with every look).

## Buying

The client sends `MarketBoardPurchaseHandler` with the listing as it saw it. The server finds the listing among the
item's current ones and checks it's still there at the same quantity and price, that the player has the gil (price ×
quantity + tax) and room for it (on a copy of the inventory); then records the sale (so the same listing can't be
bought twice), adds the item (HQ sets `item_flags` bit 1, ItemFlags.HighQuality in FFXIVClientStructs), takes the
gil, and sends the gil slot, the item's slot and `MarketBoardPurchase`. Anything wrong is a notice in chat and nothing
changes. HQ and NQ of the same item no longer share an inventory stack (`add_in_next_free_slot` checks the flags).

## The packets

All checked against retail: Bob's Chronofoil captures of 2026-09-15 (sessions 21, 26 and 27: 44 searches, 41 items
opened, 66 pages of listings, two purchases), and the 2026-10-17 bundle from Kawari for the weapons and armor tabs
(which those captures never browsed: they showed the search's category is one byte, followed by a flag). Every one of the 350 packets reads and writes back byte for byte with
these structs, except the leftovers in the page requests, which aren't kept.

| Packet | Opcode (7.57) | Size | Layout |
|---|---|---|---|
| MarketBoardOpened (client, fork) | 683 | 8 | the next request ID u32 (0 on a new session, then one more than the last search's), u32; sent as the window opens, answered with an all-zero MarketBoardItems carrying that request ID |
| RequestMarketBoardItems (client) | 921 | 136 | start index u32, request ID u8, kind u8 (0x0A first page, 0x0B the rest), **ItemSearchCategory** u8, flags u8 (1 on the Weapons and Armor tabs, 0 on Items and Housing), then 128 bytes: with flags 1 the level filter u8 and the Class/Job filter u8 (see above); then what the client doesn't clear (search text, old replies) |
| MarketBoardItems (server) | 960 | 176 | 20 × (item u32, count u32), next index u32 (0 = last page), error u32 (0; 0x70000001 on an empty reply), then the request's start index u32, request ID u8, kind u8, 2 padding |
| UnkMarketBoardRequest1 (client: open an item) | 584 | 8 | item u32, then 0x0902 (unknown) |
| MarketBoardHits (server) | 269 | 8 | 0, then how many listings (retail shows at most 100) |
| MarketBoardHistory (server) | 286 | 968 | item u32, 20 × 48-byte sales (price u32, time u32, quantity u32, HQ u8, on mannequin u8, buyer name 32, 2 padding), 4 padding |
| UnkMarketBoardRequest2 (client: a page) | 124 | 8 | start index u8, request ID u8 (counts up per item opened), 6 bytes of leftovers |
| MarketBoardOfferings (server) | 564 | 1448 | 10 × 144-byte listings, next index u8 (0 = last), start index u8, request ID u8, 5 padding |
| MarketBoardPurchaseHandler (client, fork) | 258 | 40 | retainer ID u64, listing ID u64, item u32, quantity u32, price per unit u32, tax u32, 8 unknown bytes |
| MarketBoardPurchase (server, fork) | 610 | 16 | item u32, 4 padding, quantity u32, 4 padding |

A **listing** (144 bytes): listing ID u64, retainer ID u64, retainer owner u64, crafter u64, price per unit u32 @32,
total tax u32 @36, quantity u32 @40, item u32 @44, last review u16 @48 (1–14 on retail), container u16 @50 (30000),
u16 @52, materia 5 × u16 @54, 6 bytes @64, **retainer name** 32 bytes @70, 32 zero bytes @102, HQ u8 @134, materia
count u8 @135, on mannequin u8 @136, retainer city u8 @137, stain u16 @138, 4 padding. Upstream's structs had the
history entry shifted and the offerings as raw bytes; older parsers (Dalamud's) read 6 more bytes before the name than
7.5x's listing has.

**The flow**: opening the window sends MarketBoardOpened (retail answers with an empty search result at once); the
client then sends a search page for a category (20 items per page, more as the list scrolls); opening an
item sends UnkMarketBoardRequest1, answered with MarketBoardHits and MarketBoardHistory; the client then asks for
listings 10 at a time (UnkMarketBoardRequest2 with the previous page's next index) until next index is 0. A purchase
on retail was answered with the gil slot (container 2000), an inventory transaction taking the gil, the item's slot,
then MarketBoardPurchase. Kawari sends the slots without the transaction.

**The purchase opcodes**: 765 and 665 on 2026.09.15; 258 and 610 on 2026.10.06 (FFXIVOpcodes' 7.57); 245 and 353 on
2026.10.16 (7.57h1). On 7.57h1 buying did nothing until they were found: the fork's opcodes were outstanding after the
upstream merge, and Bob's 2026-10-20 bundle shows his purchases arriving as unknown 40-byte packets (opcode 245), as
the window's opening did (319, MarketBoardOpened). See [Fork Opcodes](opcodes.md#finding-them-on-757h1).

**The marketboard object** (CustomTalk `CmnDefMarketBoardGridania_00039`) plays scene 0 with HIDE_HOTBAR |
NO_DEFAULT_CAMERA and no parameters, as retail does; the window then talks to the server through the packets above,
and the scene returns when it closes.
