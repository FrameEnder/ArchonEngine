-- Danger Zone (Lv. 18): 250.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, 250)

    return effects
end
