-- Topaz Ruin I I I (Lv. 54): 300; a stack of earth attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 2, 300)

    return effects
end
