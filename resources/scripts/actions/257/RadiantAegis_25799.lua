-- Radiant Aegis (Lv. 2): a barrier of 20% of max HP for 30 s.
local EFFECT_RADIANT_AEGIS = 2702
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_RADIANT_AEGIS, 0, DURATION)
    effects:barrier_percent_self(EFFECT_RADIANT_AEGIS, 20)

    return effects
end
