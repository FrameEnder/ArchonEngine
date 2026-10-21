-- When each character's adventurer plate was last saved (Unix seconds, 0 if never): retail sends it with the plate.
ALTER TABLE `volatile` ADD COLUMN `adventurer_plate_saved` BIGINT NOT NULL DEFAULT 0;
