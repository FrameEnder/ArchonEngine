-- Emerald Ruin I I I (Lv. 54): 180; a stack of wind attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 3, 180)

    return effects
end
