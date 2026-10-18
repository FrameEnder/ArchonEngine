-- Manaward (Lv. 30): a barrier of 30% of max HP for 20 s.
-- NOTE: barriers don't absorb damage yet; the status is applied.
local EFFECT_MANAWARD = 168
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_MANAWARD, 0, DURATION)

    return effects
end
