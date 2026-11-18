-- Cape Westwind's boss's 28875: a 2.7 s cast on the player; 535 and status 2656 for 3 s (Bob's Chronofoil capture 667a68c5 of 2026-11-16 saw it go 3.5 s later; Kawari Witness, 2026-11-18, had it on the player for 2.8 s by its quarter-second looks.) Its area is the Action sheet's; the timeline's
-- `recorded_amounts` scale it.
local POTENCY = 100
local STATUS = 2656
local DURATION = 3.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
