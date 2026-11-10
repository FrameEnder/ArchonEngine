-- Saber Dance (Lv. 76): 540 to the target, 60 % less round it; 50 Esprit.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    dncDamage(effects, player, tierPotency(player, { { 94, 540 }, { 1, 520 } }))
    effects:gauge_add("esprit", -50)

    return effects
end
