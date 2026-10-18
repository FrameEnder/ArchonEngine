-- Hallowed Ground (Lv. 50): impervious to most attacks for 10 s.
local EFFECT_HALLOWED_GROUND = 82
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_HALLOWED_GROUND, 0, DURATION)

    return effects
end
