-- Topaz Ruin (Lv. 15): 240; a stack of earth attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 2, 240)

    return effects
end
