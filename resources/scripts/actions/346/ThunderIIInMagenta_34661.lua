-- Thunder I I In Magenta (Lv. 60): 400 to the target and round it with Aetherhues II; a Subtractive Palette stack; a White Paint.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, 400)
    effects:consume_effect_stack(PCT_EFFECT_SUBTRACTIVE_PALETTE)
    pctHue(effects, player, PCT_EFFECT_AETHERHUES_II, nil)
    pctWhitePaint(effects, player)

    return effects
end
