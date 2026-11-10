-- Emerald Ruin I I (Lv. 30): 170; a stack of wind attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 3, 170)

    return effects
end
