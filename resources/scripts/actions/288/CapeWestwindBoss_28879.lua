-- Cape Westwind's boss's 28879: instant, on the player, as the last part begins: status 3134 on them, which the run
-- never saw end (71 s to his death; ten minutes here) (Bob's Chronofoil capture 667a68c5 of 2026-11-16, one run.) What the status does isn't known.
local STATUS = 3134
local DURATION = 600.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
