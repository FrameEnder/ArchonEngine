-- Shadow Wall (Lv. 38): 30 % less damage taken, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(DRK_EFFECT_SHADOW_WALL, 0, 15.0)

    return effects
end
