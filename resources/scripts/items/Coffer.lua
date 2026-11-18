-- Gear coffers (ItemAction 388, action 4647: the "Level N Weapon Coffer"s and the rest): one class's weapon of what's
-- inside, chosen by the current class or job when it's opened, the coffer used up (`player:open_coffer`, which refuses
-- in instanced content and when the class has nothing in it.) What a coffer holds is the server's: the item's
-- AdditionalData (1000238 for the Level 18 coffer) links to nothing in the client's sheets.
--
-- Each coffer: its sets of pieces by item ID, the first piece deciding whether a class gets the set (its
-- ClassJobCategory); a gladiator's sword comes with its shield. The contents are as the FFXIV Console Games Wiki lists
-- them (each coffer's page, 2026-10-30: every "Level N Weapon Coffer", 31329-31337, 31691, 31696, 31701), the item IDs
-- from the game's Item sheet (each name matched one item, of the class its place says); retail's own opening was
-- recorded for the Level 18 coffer only (a gladiator's.) The level 50 coffers have the scholar's book on its own,
-- after the arcanist's and summoner's.

COFFERS = {
    -- 31329: the level 18 weapon coffer.
    [31329] = {
        { 1611, 2241 }, -- GLA PLD, the sword with its shield
        { 1758 }, -- MRD WAR
        { 1827 }, -- LNC DRG
        { 1688 }, -- PGL MNK
        { 8662 }, -- ROG NIN
        { 1897 }, -- ARC BRD
        { 2111 }, -- THM BLM
        { 2149 }, -- ACN SMN SCH
        { 2002 }, -- CNJ WHM
    },
    -- 31330: the level 21 weapon coffer.
    [31330] = {
        { 1614, 2242 }, -- GLA PLD, the sword with its shield
        { 1764 }, -- MRD WAR
        { 1833 }, -- LNC DRG
        { 1694 }, -- PGL MNK
        { 8663 }, -- ROG NIN
        { 1899 }, -- ARC BRD
        { 2116 }, -- THM BLM
        { 2151 }, -- ACN SMN SCH
        { 2010 }, -- CNJ WHM
    },
    -- 31331: the level 24 weapon coffer.
    [31331] = {
        { 1625, 2246 }, -- GLA PLD, the sword with its shield
        { 1766 }, -- MRD WAR
        { 1836 }, -- LNC DRG
        { 1697 }, -- PGL MNK
        { 9193 }, -- ROG NIN
        { 1905 }, -- ARC BRD
        { 2118 }, -- THM BLM
        { 2156 }, -- ACN SMN SCH
        { 2011 }, -- CNJ WHM
    },
    -- 31332: the level 28 weapon coffer.
    [31332] = {
        { 1627, 2250 }, -- GLA PLD, the sword with its shield
        { 1769 }, -- MRD WAR
        { 1838 }, -- LNC DRG
        { 1699 }, -- PGL MNK
        { 8664 }, -- ROG NIN
        { 1908 }, -- ARC BRD
        { 2069 }, -- THM BLM
        { 2160 }, -- ACN SMN SCH
        { 1967 }, -- CNJ WHM
    },
    -- 31333: the level 34 weapon coffer.
    [31333] = {
        { 1637, 2260 }, -- GLA PLD, the sword with its shield
        { 1778 }, -- MRD WAR
        { 1847 }, -- LNC DRG
        { 1708 }, -- PGL MNK
        { 8665 }, -- ROG NIN
        { 1917 }, -- ARC BRD
        { 2123 }, -- THM BLM
        { 2166 }, -- ACN SMN SCH
        { 2023 }, -- CNJ WHM
    },
    -- 31334: the level 36 weapon coffer.
    [31334] = {
        { 1639, 2262 }, -- GLA PLD, the sword with its shield
        { 1781 }, -- MRD WAR
        { 1850 }, -- LNC DRG
        { 1711 }, -- PGL MNK
        { 8666 }, -- ROG NIN
        { 1920 }, -- ARC BRD
        { 2078 }, -- THM BLM
        { 2169 }, -- ACN SMN SCH
        { 2026 }, -- CNJ WHM
    },
    -- 31335: the level 40 weapon coffer.
    [31335] = {
        { 1650, 2270 }, -- GLA PLD, the sword with its shield
        { 1786 }, -- MRD WAR
        { 1855 }, -- LNC DRG
        { 1716 }, -- PGL MNK
        { 9209 }, -- ROG NIN
        { 1925 }, -- ARC BRD
        { 2083 }, -- THM BLM
        { 2174 }, -- ACN SMN SCH
        { 1974 }, -- CNJ WHM
    },
    -- 31336: the level 45 weapon coffer.
    [31336] = {
        { 1659, 2287 }, -- GLA PLD, the sword with its shield
        { 1796 }, -- MRD WAR
        { 1867 }, -- LNC DRG
        { 1728 }, -- PGL MNK
        { 8667 }, -- ROG NIN
        { 1937 }, -- ARC BRD
        { 2094 }, -- THM BLM
        { 2185 }, -- ACN SMN SCH
        { 1981 }, -- CNJ WHM
    },
    -- 31337: the level 49 weapon coffer.
    [31337] = {
        { 1663, 2296 }, -- GLA PLD, the sword with its shield
        { 1803 }, -- MRD WAR
        { 1872 }, -- LNC DRG
        { 1733 }, -- PGL MNK
        { 8668 }, -- ROG NIN
        { 1942 }, -- ARC BRD
        { 2133 }, -- THM BLM
        { 2189 }, -- ACN SMN SCH
        { 2044 }, -- CNJ WHM
    },
    -- 31691: the level 50 weapon coffer (IL 70).
    [31691] = {
        { 1666, 2302 }, -- GLA PLD, the sword with its shield
        { 1806 }, -- MRD WAR
        { 1875 }, -- LNC DRG
        { 1736 }, -- PGL MNK
        { 9225 }, -- ROG NIN
        { 1945 }, -- ARC BRD
        { 2136 }, -- THM BLM
        { 2193 }, -- ACN SMN
        { 2048 }, -- CNJ WHM
        { 2194 }, -- SCH
    },
    -- 31696: the level 50 weapon coffer (IL 90).
    [31696] = {
        { 1679, 2311 }, -- GLA PLD, the sword with its shield
        { 1818 }, -- MRD WAR
        { 1888 }, -- LNC DRG
        { 1748 }, -- PGL MNK
        { 9241 }, -- ROG NIN
        { 1957 }, -- ARC BRD
        { 2141 }, -- THM BLM
        { 2217 }, -- ACN SMN
        { 2054 }, -- CNJ WHM
        { 2218 }, -- SCH
    },
    -- 31701: the level 50 weapon coffer (IL 110).
    [31701] = {
        { 7351, 7350 }, -- GLA PLD, the sword with its shield
        { 7353 }, -- MRD WAR
        { 7354 }, -- LNC DRG
        { 7352 }, -- PGL MNK
        { 9247 }, -- ROG NIN
        { 7355 }, -- ARC BRD
        { 7357 }, -- THM BLM
        { 7358 }, -- ACN SMN
        { 7356 }, -- CNJ WHM
        { 7359 }, -- SCH
    },
}

-- Coffers that hold one thing out of many, the same for every class (`player:open_loot_coffer`): each row is
-- { item, how many, 1 if it's high quality, how often it came }.
--
-- 52450, Keybound Brawler's coffer (docs/src/fork/events.md): the 40 openings in Connor's Chronofoil captures of
-- 2026-11-03 to 11-09 and what each gave (ActorControlSelf 527 `[1232, 0, item]`, a high-quality one as the item ID
-- plus 1,000,000; `[1233, 0, 41708, 3]` for the three of 41708.) Retail's own table and odds aren't known: 30 things
-- in 40 openings says there are more than these.
LOOT_COFFERS = {
    [52450] = {
        { 5827, 1, 0, 4 },
        { 52447, 1, 0, 4 },
        { 41708, 3, 0, 3 },
        { 39562, 1, 0, 2 },
        { 46253, 1, 1, 2 },
        { 7975, 1, 0, 1 },
        { 8143, 1, 0, 1 },
        { 13114, 1, 0, 1 },
        { 13115, 1, 0, 1 },
        { 13116, 1, 0, 1 },
        { 13711, 1, 0, 1 },
        { 13714, 1, 0, 1 },
        { 17025, 1, 0, 1 },
        { 17954, 1, 0, 1 },
        { 35561, 1, 0, 1 },
        { 40404, 1, 0, 1 },
        { 40409, 1, 0, 1 },
        { 41593, 1, 0, 1 },
        { 41594, 1, 0, 1 },
        { 41601, 1, 0, 1 },
        { 41610, 1, 0, 1 },
        { 41762, 1, 0, 1 },
        { 41769, 1, 0, 1 },
        { 44977, 1, 0, 1 },
        { 44987, 1, 0, 1 },
        { 46254, 1, 1, 1 },
        { 49239, 1, 1, 1 },
        { 49240, 1, 1, 1 },
        { 49243, 1, 1, 1 },
        { 49248, 1, 1, 1 },
    },
}

-- Coffers that hold the same for whoever opens them (`player:open_coffer`'s third argument): a job quest's attire.
-- 20648, the Cleric's Attire Coffer (the White Mage's level 50 quest, Heart of the Forest): Connor's Chronofoil
-- capture e834e436 has it opened by a White Mage, action 4647 as every gear coffer's, and 1.65 s after its
-- ActionEffect items 6074, 6075, 6076, 6077 and 6078 in that order, into the Armoury Chest's head, body, hands, legs
-- and feet. The other jobs' coffers aren't in a capture yet.
FIXED_COFFERS = {
    [20648] = { 6074, 6075, 6076, 6077, 6078 },
}

function doAction(player, item_id)
    local effects = EffectsBuilder()
    local loot = LOOT_COFFERS[item_id]
    if loot ~= nil then
        player:open_loot_coffer(item_id, loot)
        return effects
    end
    local fixed = FIXED_COFFERS[item_id]
    if fixed ~= nil then
        player:open_coffer(item_id, { fixed }, true)
        return effects
    end
    local contents = COFFERS[item_id]
    if contents == nil then
        player:send_message("What coffer " .. item_id .. " holds isn't known yet")
        return effects
    end
    player:open_coffer(item_id, contents)
    return effects
end
