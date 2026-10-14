-- What battle NPCs drop when they're defeated, by BNpcName ID. Each player who fought it rolls once.
--
-- The items come from Garland Tools' monster data (garlandtools.org/db, the "drops" of each mob), which is gathered by
-- players. If a monster has more than one possible drop, one of them is picked at random.
--
-- How often, and how many, come from Kawari Recorder's recording of retail where there's enough of it (CHANCES below);
-- the rest drop DROP_CHANCE percent of the time, one at a time, which is a guess.
--
-- So far this covers the monsters in the Central Shroud (fst_f1). Monsters Garland lists no drops for aren't here.

DROP_CHANCE = 50 -- percent

-- Recorded on retail: { percent of kills that dropped something, how many each time }.
CHANCES = {
    [37] = { 17, 2 },  -- Ground Squirrel: Animal Skin x2 from 1 of 6 kills
    [120] = { 67, 3 }, -- Anole: Animal Hide x3 from 4 of 6 kills
    [21] = { 11, 1 },  -- Trickster Imp: Imp Wing from 1 of 9 kills
}

DROPS = {
    [20] = { 5432 },
    [21] = { 5552 },
    [22] = { 5548 },
    [37] = { 5291 },
    [39] = { 5495 },
    [41] = { 5492 },
    [43] = { 5492 },
    [47] = { 4794 },
    [48] = { 4794 },
    [54] = { 4850 },
    [56] = { 7, 13 },
    [117] = { 6, 12 },
    [120] = { 5292 },
    [130] = { 4767, 5292, 5310, 5322 },
    [131] = { 5229 },
    [195] = { 5292 },
    [196] = { 5291 },
    [221] = { 5292 },
    [238] = { 5554 },
    [2057] = { 5554 },
    [2190] = { 12 },
}

function onBNpcKill(name_id, level, player)
    local items = DROPS[name_id]
    if items == nil then
        return
    end

    local chance, quantity = DROP_CHANCE, 1
    if CHANCES[name_id] ~= nil then
        chance, quantity = CHANCES[name_id][1], CHANCES[name_id][2]
    end
    if math.random(1, 100) > chance then
        return
    end

    player:add_item(items[math.random(1, #items)], quantity)
end
