-- Gravity (Lv. 45): 120 to the target and round it.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, 120)

    return effects
end
