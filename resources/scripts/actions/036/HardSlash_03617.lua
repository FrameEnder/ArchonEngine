-- Hard Slash (Lv. 1): 300.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drkDamage(effects, player, tierPotency(player, { { 94, 300 }, { 84, 180 } }))
    drkWeaponskill(effects, player)

    return effects
end
