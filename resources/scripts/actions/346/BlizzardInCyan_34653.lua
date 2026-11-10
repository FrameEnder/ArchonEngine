-- Blizzard In Cyan (Lv. 60): 860; a Subtractive Palette stack; Aetherhues.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 860 }, { 84, 750 }, { 74, 680 }, { 1, 600 } }))
    effects:consume_effect_stack(PCT_EFFECT_SUBTRACTIVE_PALETTE)
    pctHue(effects, player, nil, PCT_EFFECT_AETHERHUES)

    return effects
end
