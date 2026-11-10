-- Keen Edge (Lv. 1): 300.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, tierPotency(player, { { 94, 300 }, { 84, 200 } }))

    return effects
end
