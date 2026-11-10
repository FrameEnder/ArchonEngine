-- Hakaze (Lv. 1): 200; 5 Kenki.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, tierPotency(player, { { 66, 200 }, { 1, 180 } }))
    samKenki(effects, player, 5)

    return effects
end
