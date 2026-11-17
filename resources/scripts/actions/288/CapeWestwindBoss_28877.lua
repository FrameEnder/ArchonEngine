-- Cape Westwind's boss's 28877: a 5.7 s cast on himself; 1386 and 1378 to the player (twice in four) (Bob's Chronofoil capture 667a68c5 of 2026-11-16, one run.) Its area is the Action sheet's; the timeline's
-- `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
