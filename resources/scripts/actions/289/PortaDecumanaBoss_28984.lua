-- The Porta Decumana's first boss's 28984: a 3.7 s cast on itself 235.7 s in: status 3012 on it (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each), with no timer (Kawari Witness,
-- 2026-11-18). What the status does isn't known.

local STATUS = 3012
local DURATION = 0.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
