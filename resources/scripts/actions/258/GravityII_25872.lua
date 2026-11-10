-- Gravity I I (Lv. 82): 140 to the target and round it.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, 140)

    return effects
end
