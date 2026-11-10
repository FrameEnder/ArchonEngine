-- Summon Emerald (Lv. 22): Emerald Carbuncle's Glittering Emerald, 400; four wind attunement; needs the Emerald arcanum.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnSummon(effects, player, 4, 3, 4, 400)

    return effects
end
