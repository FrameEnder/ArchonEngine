-- Stone I I In Yellow (Lv. 60): 380 to the target and round it with Aetherhues; a Subtractive Palette stack; Aetherhues II.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, 380)
    effects:consume_effect_stack(PCT_EFFECT_SUBTRACTIVE_PALETTE)
    pctHue(effects, player, PCT_EFFECT_AETHERHUES, PCT_EFFECT_AETHERHUES_II)

    return effects
end
