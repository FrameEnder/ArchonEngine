-- What the character keeps of the Gold Saucer: the Mini Cactpot tickets bought today and the Jumbo Cactpot tickets of
-- the week (see servers/world/src/gold_saucer.rs, GoldSaucerState.)
ALTER TABLE `unlock` ADD COLUMN `gold_saucer` TEXT NOT NULL DEFAULT '{}';
