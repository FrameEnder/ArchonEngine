-- Bloodbath (Lv. 12) - role action: physical damage dealt heals you for 20% of it, 20 s.
-- NOTE: the healing isn't implemented; the status is applied.
local EFFECT_BLOODBATH = 84
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_BLOODBATH, 0, DURATION)

    return effects
end
