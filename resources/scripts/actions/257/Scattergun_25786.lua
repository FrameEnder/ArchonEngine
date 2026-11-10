-- Scattergun (Lv. 82): 130 to the enemies in a cone; 10 Heat.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 130, true)
    mchHeat(effects, player, 10)

    return effects
end
