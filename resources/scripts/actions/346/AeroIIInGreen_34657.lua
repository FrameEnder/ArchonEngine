-- Aero I I In Green (Lv. 35): 200 to the target and round it with Aetherhues; Aetherhues II.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 200 }, { 74, 170 }, { 1, 140 } }))
    pctHue(effects, player, PCT_EFFECT_AETHERHUES, PCT_EFFECT_AETHERHUES_II)

    return effects
end
