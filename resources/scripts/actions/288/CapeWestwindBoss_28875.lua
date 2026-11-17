-- Cape Westwind's boss's 28875: a 2.7 s cast on the player; 535 and status 2656, which he took off 3.5 s later (Bob's Chronofoil capture 667a68c5 of 2026-11-16, one run.) Its area is the Action sheet's; the timeline's
-- `recorded_amounts` scale it.
local POTENCY = 100
local STATUS = 2656
local DURATION = 3.5

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
