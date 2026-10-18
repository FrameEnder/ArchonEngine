-- Rampart (Lv. 8) - role action: damage taken -20% for 20 s.
local EFFECT_RAMPART = 1191
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_RAMPART, 0, DURATION)

    return effects
end
