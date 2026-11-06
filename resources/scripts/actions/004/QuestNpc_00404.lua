-- Celeris as the enemy Eyes on Me's search brings out uses it (BNpcBase 1902, quest 518): a cast on itself that gives
-- it status 8 for 30 s, no damage (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06, once in each.)
local STATUS = 8
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, 0, DURATION)

    return effects
end
