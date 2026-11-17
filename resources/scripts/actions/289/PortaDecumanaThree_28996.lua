-- The Porta Decumana's base 14588's 28996: a 4.7 s cast from the first arena's middle: 1593 and 1715 to the players,
-- 436 to 1643 to the members, everyone (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of
-- 2026-11-17, one run each.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 600

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
