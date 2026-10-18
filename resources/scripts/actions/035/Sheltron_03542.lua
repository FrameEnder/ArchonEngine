-- Sheltron (Lv. 35): damage taken -15% for 6 s.
-- NOTE: the job gauge isn't implemented, so it doesn't cost Oath (50).
local EFFECT_SHELTRON = 728
local DURATION = 6.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_SHELTRON, 0, DURATION)

    return effects
end
