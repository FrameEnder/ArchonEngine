-- The Porta Decumana's base 14589's 28999: a 4.7 s cast from an edge spot, facing the middle: 546 to 589 to the
-- players and 208 to 682 to the members, everyone both times (Connor's Chronofoil capture af05916c of 2026-11-09 and
-- Bob's 9ba7d981 of 2026-11-17, one run each.) Its area is the Action sheet's; the timeline's `recorded_amounts`
-- scale it.

local POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
