-- Form Shift (Lv. 52): Formless Fist (30 s): the next form weaponskill ignores form.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(MNK_EFFECT_FORMLESS_FIST, 0, 30.0)

    return effects
end
