-- Split Shot (Lv. 1): 140; 5 Heat.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 140)
    mchHeat(effects, player, 5)

    return effects
end
