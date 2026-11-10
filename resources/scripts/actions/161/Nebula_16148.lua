-- Nebula (Lv. 38): 30 % less damage taken, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(GNB_EFFECT_NEBULA, 0, 15.0)

    return effects
end
