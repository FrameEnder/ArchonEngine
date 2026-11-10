-- Drill (Lv. 58): 660.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 660)

    return effects
end
