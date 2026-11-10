-- Ruby Ruin I I I (Lv. 54): 410; a stack of fire attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 1, 410)

    return effects
end
