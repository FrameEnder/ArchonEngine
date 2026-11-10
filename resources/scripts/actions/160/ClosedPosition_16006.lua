-- Closed Position (Lv. 60): the dance partner, until used again.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(DNC_EFFECT_DANCE_PARTNER, 0, 0.0)
    effects:gain_effect_self(DNC_EFFECT_CLOSED_POSITION, 0, 0.0)

    return effects
end
