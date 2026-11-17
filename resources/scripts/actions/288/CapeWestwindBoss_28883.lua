-- Cape Westwind's boss's 28883: a 4.7 s cast on himself every 13.15 s in the last part, a stack of status 3008 more on him each time; 507-626 to the player (Bob's Chronofoil capture 667a68c5 of 2026-11-16, one run.) Its area is the Action sheet's; the timeline's
-- `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
