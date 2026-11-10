-- Zoe (Lv. 56): the next healing spell 50 % stronger, 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SGE_EFFECT_ZOE, 0, 30.0)

    return effects
end
