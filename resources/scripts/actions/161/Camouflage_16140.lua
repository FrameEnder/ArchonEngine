-- Camouflage (Lv. 6): 50 % more parries and 10 % less damage taken, 20 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(GNB_EFFECT_CAMOUFLAGE, 0, 20.0)

    return effects
end
