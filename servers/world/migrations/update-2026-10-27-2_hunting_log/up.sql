-- The Hunting Log's progress: a list of { index, rank, counts (40 kills: entry * 4 + target), completed (five bits an
-- entry) }, one per log begun. Missing is a log not begun.
ALTER TABLE `unlock` ADD COLUMN `hunting_log` TEXT NOT NULL DEFAULT '[]';
