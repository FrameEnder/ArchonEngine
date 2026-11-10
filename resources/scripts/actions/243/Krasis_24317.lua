-- Krasis (Lv. 86): 20 % more healing received, 10 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SGE_EFFECT_KRASIS, 0, 10.0)

    return effects
end
