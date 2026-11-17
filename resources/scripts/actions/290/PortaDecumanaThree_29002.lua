-- The Porta Decumana's base 14591's 29002: a 4.7 s cast from the first arena's middle: 1625 and 1735 to the players,
-- 439 to 1607 to the members, everyone (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of
-- 2026-11-17, one run each.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 600

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
