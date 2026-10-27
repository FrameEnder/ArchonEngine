-- Like Mother, Like Daughter's sprite on your side (quest battle 56, BNpcBase 1692): a 1.2 s cast on itself every
-- 34.5 s giving itself status 199 with param 50 (Bob's Chronofoil capture b23b91ef, 2026-10-26, its ActionEffects).
-- How long the status lasts wasn't seen: 30 s is a guess. What 199 does isn't scripted.
local STATUS = 199
local PARAM = 50
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
