-- The chocobo companion's skill points, command (BuddyAction: 3 Follow ... 7 Healer Stance), favourite feed and barding
-- (BuddyEquip rows: head, body, legs), which the client gets in PlayerSetup.
ALTER TABLE `buddy` ADD COLUMN `skill_points` INTEGER NOT NULL DEFAULT 0;
ALTER TABLE `buddy` ADD COLUMN `active_command` INTEGER NOT NULL DEFAULT 4;
ALTER TABLE `buddy` ADD COLUMN `favorite_feed` INTEGER NOT NULL DEFAULT 0;
ALTER TABLE `buddy` ADD COLUMN `barding` TEXT NOT NULL DEFAULT '[0,0,0]';
