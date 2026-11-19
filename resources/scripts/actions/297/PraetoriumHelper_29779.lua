-- The Praetorium's helpers' 29779: a 5.7 s cast in the middle of the last boss's room, the way the boss faces
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18,
-- five runs.) Its area is the Action sheet's; it hit no one in the runs, so its damage here is a guess.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
