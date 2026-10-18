-- Iron Will (Lv. 10): the tank stance, until used again.
local EFFECT_IRON_WILL = 79

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    toggleStance(player, effects, EFFECT_IRON_WILL)

    return effects
end
