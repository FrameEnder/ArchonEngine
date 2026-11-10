-- Summon Topaz (Lv. 15): Topaz Carbuncle's Glittering Topaz, 400; four earth attunement; needs the Topaz arcanum.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnSummon(effects, player, 2, 2, 4, 400)

    return effects
end
