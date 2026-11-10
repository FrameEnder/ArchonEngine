-- Emerald Outburst (Lv. 26): 70 to the target and round it; a stack of wind attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 3, 70)

    return effects
end
