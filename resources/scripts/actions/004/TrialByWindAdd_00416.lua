-- Trial by Wind's last arrival (quest battle 54, BNpcBase 1660): a 2.2 s cast on itself 5 s after it appears, giving
-- itself status 57 with param 50 (Bob's Chronofoil capture b23b91ef, 2026-10-26, its ActionEffect.) How long the
-- status lasts wasn't seen (the fight was over 8 s later): 30 s is a guess. What 57 does isn't scripted.
local STATUS = 57
local PARAM = 50
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
