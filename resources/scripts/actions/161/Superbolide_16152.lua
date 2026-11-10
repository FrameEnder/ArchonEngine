-- Superbolide (Lv. 50): all but immune, 10 s (the half of the HP it costs isn't taken).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(GNB_EFFECT_SUPERBOLIDE, 0, 10.0)

    return effects
end
