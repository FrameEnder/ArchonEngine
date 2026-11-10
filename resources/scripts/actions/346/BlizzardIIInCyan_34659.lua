-- Blizzard I I In Cyan (Lv. 60): 360 to the target and round it; a Subtractive Palette stack; Aetherhues.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, 360)
    effects:consume_effect_stack(PCT_EFFECT_SUBTRACTIVE_PALETTE)
    pctHue(effects, player, nil, PCT_EFFECT_AETHERHUES)

    return effects
end
