-- Comet In Black (Lv. 90): 940 to the target, 65 % less round it; a White Paint (the Black one) and Monochrome Tones.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, 940, 65)
    effects:gauge_add("white_paint", -1)
    effects:consume_effect(PCT_EFFECT_MONOCHROME_TONES)
    pctHue(effects, player, nil, nil)

    return effects
end
