-- Expedient (Lv. 90): the party near is faster (10 s) and takes 10 % less damage (Desperate Measures, 20 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SCH_EFFECT_DESPERATE_MEASURES, 0, 20.0)

    return effects
end
