-- Smudge (Lv. 20): a dash forward (movement isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(PCT_EFFECT_SMUDGE, 0, 5.0)

    return effects
end
