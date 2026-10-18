-- Sleep (Lv. 10) - role action: the target and enemies near it fall asleep for 30 s.
local SLEEP_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_SLEEP, 0, SLEEP_DURATION)

    return effects
end
