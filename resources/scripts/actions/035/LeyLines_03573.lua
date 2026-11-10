-- Ley Lines (Lv. 52): a circle (20 s) where spells cast and recast 15 % faster (Circle of Power; the haste isn't done.)
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(737, 0, 20.0)
    effects:gain_effect_self(738, 0, 20.0)

    return effects
end
