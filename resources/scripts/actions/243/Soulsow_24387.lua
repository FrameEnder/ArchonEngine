-- Soulsow (Lv. 82): Soulsow (Harvest Moon), until used.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(RPR_EFFECT_SOULSOW, 0, 0.0)

    return effects
end
