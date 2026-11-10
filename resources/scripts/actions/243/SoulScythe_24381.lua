-- Soul Scythe (Lv. 65): 180 to all enemies near; 50 Soul.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 180)
    rprSoul(effects, player, 50)

    return effects
end
