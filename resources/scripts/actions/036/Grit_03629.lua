-- Grit (Lv. 10): the tank stance, until used again.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    toggleStance(player, effects, DRK_EFFECT_GRIT)

    return effects
end
