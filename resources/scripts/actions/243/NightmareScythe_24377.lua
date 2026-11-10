-- Nightmare Scythe (Lv. 45): 120 to all enemies near, 180 after Spinning Scythe with 10 Soul.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        rprDamage(effects, player, 180)
        rprSoul(effects, player, 10)
    else
        rprDamage(effects, player, 120)
    end

    return effects
end
