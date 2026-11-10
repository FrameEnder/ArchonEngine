-- Winged Muse (Lv. 30): 800 to the target, 70 % less round it; the Wings depicted, and with the Pom the Moogle Portrait.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMuse(effects, player, 2, 2, tierPotency(player, { { 84, 800 }, { 54, 700 }, { 1, 600 } }))

    return effects
end
