-- Fire In Red (Lv. 1): 490; Aetherhues (30 s).
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 490 }, { 84, 420 }, { 74, 380 }, { 54, 310 }, { 1, 280 } }))
    pctHue(effects, player, nil, PCT_EFFECT_AETHERHUES)

    return effects
end
