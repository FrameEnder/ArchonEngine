-- The Porta Decumana's first boss's 28978: a 4.7 s cast on itself as base 14591's 29002 begins. Status 3011 is on it
-- from then until its 28995 takes it off (the action's effect names that status; its coming wasn't in the action's
-- effects, so that this cast gives it is a guess) (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's
-- 9ba7d981 of 2026-11-17, one run each.) What the status does isn't known.

local STATUS = 3011
local DURATION = 3600.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
