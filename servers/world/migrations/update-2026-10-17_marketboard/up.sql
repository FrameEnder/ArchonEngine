-- The marketboard (docs/src/fork/marketboard.md). Kawari Web's Marketboard page edits the first three tables; the world
-- server reads them whenever the board is used, so changes apply at once.

-- Price rates. `scope` is 'tab:N' for a marketboard tab (ItemSearchCategory.Category: 1 weapons, 2 armor, 3 items,
-- 4 housing) or 'category:N' for one ItemSearchCategory. A category row's NULLs mean "the tab's value".
-- An item's price: per_level × its level (equip level or item level, level_source) + its rarity's bonus; HQ multiplies
-- the rarity bonus by hq_multiplier.
CREATE TABLE marketboard_rate (
    scope TEXT PRIMARY KEY NOT NULL,
    per_level INTEGER,
    level_source TEXT,
    rarity_white INTEGER,
    rarity_pink INTEGER,
    rarity_green INTEGER,
    rarity_blue INTEGER,
    rarity_purple INTEGER,
    hq_multiplier INTEGER
);
INSERT INTO marketboard_rate VALUES ('tab:1', 100, 'equip', 0, 2000, 4000, 8000, 20000, 2);
INSERT INTO marketboard_rate VALUES ('tab:2', 100, 'equip', 0, 2000, 4000, 8000, 20000, 2);
INSERT INTO marketboard_rate VALUES ('tab:3', 3, 'item', 0, 20, 40, 80, 200, 2);
INSERT INTO marketboard_rate VALUES ('tab:4', 50, 'item', 0, 2000, 4000, 8000, 20000, 2);

-- One item's own prices, in gil per unit; NULL means the computed one.
CREATE TABLE marketboard_item (
    item_id INTEGER PRIMARY KEY NOT NULL,
    nq_price INTEGER,
    hq_price INTEGER
);

-- How many listings each item has, how many of them HQ, the largest stack, and the tax buyers pay (percent.)
CREATE TABLE marketboard_setting (
    key TEXT PRIMARY KEY NOT NULL,
    value INTEGER NOT NULL
);
INSERT INTO marketboard_setting VALUES ('listings_per_item', 50);
INSERT INTO marketboard_setting VALUES ('hq_listings', 10);
INSERT INTO marketboard_setting VALUES ('max_stack', 99);
INSERT INTO marketboard_setting VALUES ('tax_percent', 5);

-- Every purchase: the item's history, and the listings that are gone (a new one takes each one's place.)
CREATE TABLE marketboard_sale (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    item_id INTEGER NOT NULL,
    listing_id BIGINT NOT NULL,
    hq INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    price_per_unit INTEGER NOT NULL,
    tax INTEGER NOT NULL,
    buyer_content_id BIGINT NOT NULL,
    buyer_name TEXT NOT NULL,
    sold_at BIGINT NOT NULL
);
CREATE INDEX marketboard_sale_item ON marketboard_sale(item_id, id DESC);
