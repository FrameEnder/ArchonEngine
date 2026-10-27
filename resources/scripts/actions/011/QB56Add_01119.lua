-- Like Mother, Like Daughter's 1686 (quest battle 56, BNpcBase 1686): a 3.7 s cast on a conjurer that gave it status
-- 9 with param 60 and no damage (Bob's Chronofoil capture b23b91ef, 2026-10-26, the lost run, its ActionEffect). How
-- long the status lasts wasn't seen: 15 s is a guess. What 9 does isn't scripted.
local STATUS = 9
local PARAM = 60
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
