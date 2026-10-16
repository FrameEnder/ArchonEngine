-- Remember each character's completed achievements, the last five completed, and their adventurer plate's portrait.
ALTER TABLE `unlock` ADD COLUMN `achievements` TEXT NOT NULL DEFAULT '[]';
ALTER TABLE `unlock` ADD COLUMN `achievement_history` TEXT NOT NULL DEFAULT '[]';
ALTER TABLE `volatile` ADD COLUMN `portrait` TEXT NOT NULL DEFAULT '[]';
