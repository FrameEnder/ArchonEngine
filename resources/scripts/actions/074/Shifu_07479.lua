-- Shifu (Lv. 18): 140, 300 after Hakaze / Gyofu with Fuka (13 % quicker, 40 s; the haste isn't done) and 5 Kenki.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samStep(effects, player, in_combo, tierPotency(player, { { 94, 140 }, { 66, 120 } }), 300, 0, 5, SAM_EFFECT_FUKA)

    return effects
end
