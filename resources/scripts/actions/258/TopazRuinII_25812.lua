-- Topaz Ruin I I (Lv. 30): 270; a stack of earth attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 2, 270)

    return effects
end
