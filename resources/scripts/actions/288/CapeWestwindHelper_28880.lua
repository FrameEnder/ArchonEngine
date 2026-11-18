-- Cape Westwind's helper's 28880: a 0.2 s cast on the player, 0.9 s after the boss's 28879, from 2.5 yalms in front of
-- him: effect 32 with the Attract sheet's row 17 (`20 00 00 00 00 00 11 00`), which drew them to where the helper
-- stood (10.0 yalms off it to 0.1 in 0.9 s, by their own client's position updates: Bob's Chronofoil captures
-- 667a68c5 of 2026-11-16 and 178b0de0 of 2026-11-18, and two more runs.) Read as the Knockback sheet's row 17 until
-- 2026-11-18, which pushed the player away.
local ATTRACT = 17

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:attract(ATTRACT)

    return effects
end
