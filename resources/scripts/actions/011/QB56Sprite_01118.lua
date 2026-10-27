-- Like Mother, Like Daughter's wind sprite on your side (quest battle 56, BNpcBase 1691): a 3.7 s cast on an enemy
-- every 12.6 s that gave it status 17 with param 50 and no damage (Bob's Chronofoil capture b23b91ef, 2026-10-26, its
-- ActionEffects). How long the status lasts wasn't seen: 10 s is a guess. What 17 does isn't scripted.
local STATUS = 17
local PARAM = 50
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
