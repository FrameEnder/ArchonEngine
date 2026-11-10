-- Damnation (Lv. 92): 40 % less damage for 15 s, 55 back on each physical hit; Primeval Impulse after.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(3832, 0, 15.0)

    return effects
end
