-- Castrum Meridianum's helpers' 28781: a 5 s cast from the middle of the second boss's room (a circle of 10)
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) Its
-- area is the Action sheet's; it hit no one in the runs, so its damage here is a guess.

local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
