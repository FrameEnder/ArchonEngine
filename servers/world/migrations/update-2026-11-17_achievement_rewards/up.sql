-- The achievements whose item has been claimed (the Achievements window's reward, client trigger 9003 command 22):
-- a bitmask like `achievements`.
ALTER TABLE `unlock` ADD COLUMN `achievement_rewards` TEXT NOT NULL DEFAULT '[]';
