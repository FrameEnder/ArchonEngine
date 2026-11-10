-- Tengentsu (Lv. 82): 10 % less damage from the next hit, 4 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SAM_EFFECT_TENGENTSU, 0, 4.0)

    return effects
end
