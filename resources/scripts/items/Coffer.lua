-- Gear coffers (ItemAction 388, action 4647: the "Level N Weapon Coffer"s and the rest): one class's weapon of what's
-- inside, chosen by the current class or job when it's opened, the coffer used up (`player:open_coffer`, which refuses
-- in instanced content and when the class has nothing in it.) What a coffer holds is the server's: the item's
-- AdditionalData (1000238 for the Level 18 coffer) links to nothing in the client's sheets.
--
-- Each coffer: its sets of pieces by item ID, the first piece deciding whether a class gets the set (its
-- ClassJobCategory); a gladiator's sword comes with its shield. The contents are as the FFXIV Console Games Wiki lists
-- them (each coffer's page), the item IDs from the game's Item sheet; retail's own opening isn't recorded yet.

COFFERS = {
    -- Level 18 Weapon Coffer, from Life, Materia and Everything.
    [31329] = {
        { 1611, 2241 }, -- GLA PLD sword, with its shield
        { 1758 }, -- MRD WAR
        { 1827 }, -- LNC DRG
        { 1688 }, -- PGL MNK
        { 8662 }, -- ROG NIN
        { 1897 }, -- ARC BRD
        { 2111 }, -- THM BLM
        { 2149 }, -- ACN SMN SCH
        { 2002 }, -- CNJ WHM
    },
    -- Level 24 Weapon Coffer, from Meeting with the Resistance.
    [31331] = {
        { 1625, 2246 }, -- GLA PLD
        { 1766 }, -- MRD WAR
        { 1836 }, -- LNC DRG
        { 1697 }, -- PGL MNK
        { 9193 }, -- ROG NIN
        { 1905 }, -- ARC BRD
        { 2118 }, -- THM BLM
        { 2156 }, -- ACN SMN SCH
        { 2011 }, -- CNJ WHM
    },
}

function doAction(player, item_id)
    local effects = EffectsBuilder()
    local contents = COFFERS[item_id]
    if contents == nil then
        player:send_message("What coffer " .. item_id .. " holds isn't known yet")
        return effects
    end
    player:open_coffer(item_id, contents)
    return effects
end
