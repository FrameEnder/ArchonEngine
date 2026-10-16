-- The counters "do this N times" achievements count (Achievement Type 1), by their Key.
ALTER TABLE `unlock` ADD COLUMN `achievement_counters` TEXT NOT NULL DEFAULT '[]';
