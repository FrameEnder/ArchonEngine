-- Battle Voice (Lv. 50): direct hit rate +20% for self and nearby party members, 20 s.
local EFFECT_BATTLE_VOICE = 141
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_BATTLE_VOICE, 0, DURATION)

    return effects
end
