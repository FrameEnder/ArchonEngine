-- Cape Westwind's boss's 28879: instant, on the player, as the last part begins: status 3134 on them, with no
-- timer (Kawari Witness, 2026-11-18: gone 107 s on, as the player left; Bob's Chronofoil capture 667a68c5 of
-- 2026-11-16 never saw it end.) What the status does isn't known.
local STATUS = 3134
local DURATION = 0.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
