-- Bulwark (Lv. 52): blocks every attack for 10 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(77, 0, 10.0)

    return effects
end
