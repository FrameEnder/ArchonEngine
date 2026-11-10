-- Guardian (Lv. 92): 40 % less damage for 15 s and a barrier (barriers don't absorb yet).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(3829, 0, 15.0)
    effects:gain_effect_self(3830, 0, 15.0)

    return effects
end
