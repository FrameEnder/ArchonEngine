-- Hide (Lv. 10): Hidden until another action is used.
-- NOTE: being undetected and the slower movement aren't implemented.
local EFFECT_HIDDEN = 614

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_HIDDEN, 0, 0.0)

    return effects
end
