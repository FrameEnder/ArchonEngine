-- Emerald Disaster (Lv. 74): 90 to the target and round it; a stack of wind attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 3, 90)

    return effects
end
