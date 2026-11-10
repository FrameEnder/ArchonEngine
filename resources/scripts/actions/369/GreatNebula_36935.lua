-- Great Nebula (Lv. 92): 40 % less damage taken, 15 s (the 20 % more HP isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(GNB_EFFECT_GREAT_NEBULA, 0, 15.0)

    return effects
end
