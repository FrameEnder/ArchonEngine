-- The rank the character has with each allied society (the BeastTribe sheet's rows), a byte a society
-- (`zone_connection/allied_society.rs`.)
ALTER TABLE `unlock` ADD COLUMN `allied_society_ranks` TEXT NOT NULL DEFAULT '[]';
