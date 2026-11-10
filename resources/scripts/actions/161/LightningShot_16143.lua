-- Lightning Shot (Lv. 15): 150.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, 150)

    return effects
end
