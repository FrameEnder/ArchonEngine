-- Oblation (Lv. 82): 10 % less damage taken, 10 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(DRK_EFFECT_OBLATION, 0, 10.0)

    return effects
end
