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

-- Recorded on retail: { percent of kills that dropped something, how many each time }. Kawari Recorder's recording
-- plus Bob's Chronofoil captures of 2026-10 (counts of both together.)
CHANCES = {
    [37] = { 21, 2 },  -- Ground Squirrel: 3 of 14 kills, x2 or x3
    [120] = { 67, 3 }, -- Anole: 8 of 12 kills, x3
    [21] = { 11, 1 },  -- Trickster Imp: 2 of 18 kills
    [43] = { 20, 1 },  -- none in 4 kills (a guess below the default)
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

    -- Obtained directly, with retail's loot message ("You obtain ...").
    player:obtain_drop(items[math.random(1, #items)], quantity)
end
