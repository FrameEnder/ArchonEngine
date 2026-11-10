-- Thunder In Magenta (Lv. 60): 940 with Aetherhues II; a Subtractive Palette stack; a White Paint.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 940 }, { 84, 830 }, { 74, 760 }, { 1, 680 } }))
    effects:consume_effect_stack(PCT_EFFECT_SUBTRACTIVE_PALETTE)
    pctHue(effects, player, PCT_EFFECT_AETHERHUES_II, nil)
    pctWhitePaint(effects, player)

    return effects
end
