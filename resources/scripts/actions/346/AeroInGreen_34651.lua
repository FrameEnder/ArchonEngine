-- Aero In Green (Lv. 5): 530 with Aetherhues; Aetherhues II.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 530 }, { 84, 460 }, { 74, 420 }, { 54, 350 }, { 1, 320 } }))
    pctHue(effects, player, PCT_EFFECT_AETHERHUES, PCT_EFFECT_AETHERHUES_II)

    return effects
end
