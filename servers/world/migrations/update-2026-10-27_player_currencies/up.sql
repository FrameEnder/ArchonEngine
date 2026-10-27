-- The currencies the client counts outside the inventory (ActorControlSelf 378, PlayerCurrency), such as Achievement
-- Certificates: a list of { item_id, count }. Missing is 0.
ALTER TABLE `unlock` ADD COLUMN `player_currencies` TEXT NOT NULL DEFAULT '[]';
