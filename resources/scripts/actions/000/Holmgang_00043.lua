-- Holmgang (Lv. 42): HP can't drop below 1 for 10 s.
-- NOTE: binding the target isn't implemented.
local EFFECT_HOLMGANG = 409
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_HOLMGANG, 0, DURATION)

    return effects
end
