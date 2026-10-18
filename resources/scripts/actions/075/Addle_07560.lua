-- Addle (Lv. 8) - role action: the target deals 10% less magic and 5% less physical damage for 10 s.
local EFFECT_ADDLE = 1203
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_ADDLE, 0, DURATION)

    return effects
end
