-- Stone In Yellow (Lv. 60): 900 with Aetherhues; a Subtractive Palette stack; Aetherhues II.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 900 }, { 84, 790 }, { 74, 720 }, { 1, 640 } }))
    effects:consume_effect_stack(PCT_EFFECT_SUBTRACTIVE_PALETTE)
    pctHue(effects, player, PCT_EFFECT_AETHERHUES, PCT_EFFECT_AETHERHUES_II)

    return effects
end
