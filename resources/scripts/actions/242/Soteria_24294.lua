-- Soteria (Lv. 35): four stacks of Soteria (Kardion's heals 70 % more; not done), 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SGE_EFFECT_SOTERIA, 4, 15.0)

    return effects
end
