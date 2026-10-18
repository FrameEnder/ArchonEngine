-- Peloton (Lv. 20) - role action: movement speed +20 for self and party members within 30 yalms, 30 s.
-- NOTE: retail ends it when combat starts; that isn't implemented.
local EFFECT_PELOTON = 1199
local SPEED = 20
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_PELOTON, SPEED, DURATION)

    return effects
end
