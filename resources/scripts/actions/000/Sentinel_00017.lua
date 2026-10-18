-- Sentinel (Lv. 38): damage taken -30% for 15 s.
local EFFECT_SENTINEL = 74
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_SENTINEL, 0, DURATION)

    return effects
end
