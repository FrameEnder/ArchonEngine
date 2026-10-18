-- Retainers (docs/src/fork/retainers.md.) One row per retainer; `display_order` is their slot in the summoning bell's
-- list (0-9). `customize` is their appearance, the 26 bytes of CustomizeData in hex, and `inventory` their items as
-- JSON (crate::retainer::RetainerInventory): what they hold, wear, their gil, crystals and market listings.
CREATE TABLE retainer (
    id BIGINT PRIMARY KEY NOT NULL,
    content_id BIGINT NOT NULL,
    display_order INTEGER NOT NULL,
    name TEXT NOT NULL,
    classjob INTEGER NOT NULL,
    level INTEGER NOT NULL,
    town INTEGER NOT NULL,
    customize TEXT NOT NULL,
    voice INTEGER NOT NULL,
    inventory TEXT NOT NULL,
    hired_at BIGINT NOT NULL,
    FOREIGN KEY (content_id) REFERENCES character(content_id)
);
CREATE INDEX retainer_owner ON retainer(content_id, display_order);
