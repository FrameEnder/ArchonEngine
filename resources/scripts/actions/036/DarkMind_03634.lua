-- Dark Mind (Lv. 45): 10 % less physical and 20 % less magic damage taken, 10 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(DRK_EFFECT_DARK_MIND, 0, 10.0)

    return effects
end
