-- What battle NPCs drop when they're defeated, by BNpcName ID. Each player who fought it rolls once.
--
-- The items come from Garland Tools' monster data (garlandtools.org/db, the "drops" of each mob), which is gathered by
-- players. Neither it nor the game data says how often things drop, so DROP_CHANCE is a guess; tune it to what you see
-- on retail. If a monster has more than one possible drop, one of them is picked at random.
--
-- So far this covers the monsters in the Central Shroud (fst_f1). Monsters Garland lists no drops for aren't here.

DROP_CHANCE = 50 -- percent

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
    if items == nil or math.random(1, 100) > DROP_CHANCE then
        return
    end

    player:add_item(items[math.random(1, #items)], 1)
end
