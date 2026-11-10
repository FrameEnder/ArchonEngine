-- Slice (Lv. 1): 420; 10 Soul.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, tierPotency(player, { { 94, 420 }, { 84, 320 }, { 60, 300 }, { 1, 260 } }))
    rprSoul(effects, player, 10)

    return effects
end
