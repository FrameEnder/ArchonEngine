-- Defiance (Lv. 10): the tank stance, until used again.
local EFFECT_DEFIANCE = 91

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    toggleStance(player, effects, EFFECT_DEFIANCE)

    return effects
end
