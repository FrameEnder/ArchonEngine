-- Yukikaze (Lv. 50): 160, 340 after Hakaze / Gyofu with Setsu and 15 Kenki.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samStep(effects, player, in_combo, tierPotency(player, { { 94, 160 }, { 84, 110 } }), 340, 1, 15, nil)

    return effects
end
