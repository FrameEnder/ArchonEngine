-- The Porta Decumana's helpers' 29021: a 6.7 s cast 20 yalms north, south, east and west of the second arena's
-- middle, four at once: 135 to 248 to a player and 54 to 172 to a member a hit, all four hitting everyone (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.) Its area is the Action
-- sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 60

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
