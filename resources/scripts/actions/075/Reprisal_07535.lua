-- Reprisal (Lv. 22) - role action: enemies within 5 yalms deal 10% less damage for 10 s.
local EFFECT_REPRISAL = 1193
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_REPRISAL, 0, DURATION)

    return effects
end
