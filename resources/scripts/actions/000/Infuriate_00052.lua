-- Infuriate (Lv. 50): Nascent Chaos for 30 s.
-- NOTE: the job gauge isn't implemented, so the Beast Gauge (+50) isn't given.
local EFFECT_NASCENT_CHAOS = 1897
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_NASCENT_CHAOS, 0, DURATION)

    return effects
end
