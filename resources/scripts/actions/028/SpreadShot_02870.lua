-- Spread Shot (Lv. 18): 110 to the enemies in a cone; 5 Heat.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 110, true)
    mchHeat(effects, player, 5)

    return effects
end
