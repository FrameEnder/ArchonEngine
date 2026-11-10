-- Waxing Slice (Lv. 5): 260, 500 after Slice with 10 Soul.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        rprDamage(effects, player, 500)
        rprSoul(effects, player, 10)
    else
        rprDamage(effects, player, tierPotency(player, { { 94, 260 }, { 84, 160 }, { 60, 140 } }))
    end

    return effects
end
