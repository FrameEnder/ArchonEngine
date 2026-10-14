-- Remember which parts of the map each character has discovered.
ALTER TABLE `unlock` ADD COLUMN `map_discovery` TEXT NOT NULL DEFAULT '[]';
