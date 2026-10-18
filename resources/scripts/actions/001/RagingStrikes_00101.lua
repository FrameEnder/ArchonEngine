-- Raging Strikes (Lv. 4): damage dealt +15% for 20 s.
local EFFECT_RAGING_STRIKES = 125
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_RAGING_STRIKES, 0, DURATION)

    return effects
end
