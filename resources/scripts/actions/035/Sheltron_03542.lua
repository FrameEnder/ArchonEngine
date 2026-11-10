-- Sheltron (Lv. 35): damage taken -15% for 6 s.
-- Costs 50 Oath (the job gauge's "oath").
local EFFECT_SHELTRON = 1856 -- retail's (Bob's Chronofoil captures), 728 is an older Sheltron
local DURATION = 6.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_SHELTRON, 0, DURATION)
    effects:gauge_add("oath", -50)

    return effects
end
