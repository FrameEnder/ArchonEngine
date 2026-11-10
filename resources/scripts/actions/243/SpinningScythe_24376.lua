-- Spinning Scythe (Lv. 25): 140 to all enemies near; 10 Soul.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 140)
    rprSoul(effects, player, 10)

    return effects
end
