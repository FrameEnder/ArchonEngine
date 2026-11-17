-- Cape Westwind's boss's 28871: a 2.7 s cast on himself facing the player; landed once in four (897, blocked) (Bob's Chronofoil capture 667a68c5 of 2026-11-16, one run.) Its area is the Action sheet's; the timeline's
-- `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
