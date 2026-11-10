-- Fire I I In Red (Lv. 25): 180 to the target and round it; Aetherhues.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 180 }, { 74, 150 }, { 1, 120 } }))
    pctHue(effects, player, nil, PCT_EFFECT_AETHERHUES)

    return effects
end
