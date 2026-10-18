-- Elusive Jump (Lv. 35): jumps back, grants Enhanced Piercing Talon for 15 s.
-- NOTE: the movement is the client's.
local EFFECT_ENHANCED_PIERCING_TALON = 1870
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_ENHANCED_PIERCING_TALON, 0, DURATION)

    return effects
end
