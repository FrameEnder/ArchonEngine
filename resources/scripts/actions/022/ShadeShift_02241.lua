-- Shade Shift (Lv. 2): a barrier of 20% of max HP for 20 s.
local EFFECT_SHADE_SHIFT = 488
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_SHADE_SHIFT, 0, DURATION)
    effects:barrier_percent_self(EFFECT_SHADE_SHIFT, 20)

    return effects
end
