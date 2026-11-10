-- Hot Shot (Lv. 4): 240; 20 Battery.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 240)
    mchHeat(effects, player, 0, 20)

    return effects
end
