-- Ruby Disaster (Lv. 74): 190 to the target and round it; a stack of fire attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 1, 190)

    return effects
end
