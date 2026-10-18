-- True North (Lv. 50) - role action: positionals always count, 10 s (positionals aren't checked anyway).
local EFFECT_TRUE_NORTH = 1250
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_TRUE_NORTH, 0, DURATION)

    return effects
end
