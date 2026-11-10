-- Hells Egress (Lv. 20): a dash (movement isn't done); Enhanced Harpe and Threshold, 10 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(RPR_EFFECT_ENHANCED_HARPE, 0, 10.0)
    effects:gain_effect_self(RPR_EFFECT_THRESHOLD, 0, 10.0)

    return effects
end
