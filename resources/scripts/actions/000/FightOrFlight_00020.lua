-- Fight or Flight (Lv. 2): damage dealt +25% for 20 s.
local EFFECT_FIGHT_OR_FLIGHT = 76
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_FIGHT_OR_FLIGHT, 0, DURATION)

    return effects
end
