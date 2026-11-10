-- Emerald Rite (Lv. 72): 280; a stack of wind attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 3, 280)

    return effects
end
