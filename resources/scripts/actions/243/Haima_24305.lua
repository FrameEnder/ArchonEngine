-- Haima (Lv. 70): a barrier and five Haimatinon, 15 s (barriers don't absorb yet).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SGE_EFFECT_HAIMA, 0, 15.0)

    return effects
end
