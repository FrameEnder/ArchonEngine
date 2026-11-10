-- Infernal Slice (Lv. 30): 280, 600 after Waxing Slice with 10 Soul.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        rprDamage(effects, player, 600)
        rprSoul(effects, player, 10)
    else
        rprDamage(effects, player, tierPotency(player, { { 94, 280 }, { 84, 180 }, { 60, 140 } }))
    end

    return effects
end
