-- The recipes made at least once (the Crafting Log's completion marks, which a quick synthesis needs): the client's
-- completed recipes bitmask (CraftingLog at login, RecordCraftingLog as one is made), by recipe ID. Shorter than the
-- bitmask (as this default) is read as the rest unset.
ALTER TABLE `quest` ADD COLUMN `completed_recipes` TEXT NOT NULL DEFAULT '[]';
