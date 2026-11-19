-- The Praetorium's helpers' 28469: a 6.7 s cast, four round the first boss's room's middle, 6 yalms out: 1313 to a
-- player (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17
-- and 18, five runs.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
