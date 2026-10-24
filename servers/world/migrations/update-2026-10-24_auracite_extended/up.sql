-- An imported Auracite backup's `kawari_extended` object (the extended plugin's additions), kept whole: what Kawari
-- has a place for is also imported into the other tables, the rest waits here for a later Kawari or an export.
CREATE TABLE `auracite_extended` (
    `content_id` BIGINT NOT NULL PRIMARY KEY,
    `data` TEXT NOT NULL,
    `imported_at` BIGINT NOT NULL DEFAULT 0
);
