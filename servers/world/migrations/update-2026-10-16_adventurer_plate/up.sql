-- Remember each character's adventurer plate as the client last saved it.
ALTER TABLE `volatile` ADD COLUMN `adventurer_plate` TEXT NOT NULL DEFAULT '[]';
