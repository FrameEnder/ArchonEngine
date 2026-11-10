-- Living Dead (Lv. 50): Living Dead, 10 s (Walking Dead isn't done: HP stops at 1 meanwhile).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(DRK_EFFECT_LIVING_DEAD, 0, 10.0)

    return effects
end
