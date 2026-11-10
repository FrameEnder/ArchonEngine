-- Writhing Snap (Lv. 15): 200.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 200)

    return effects
end
