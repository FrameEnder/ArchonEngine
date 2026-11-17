-- The Porta Decumana's base 14593's 29019: a 3.7 s cast across the arena from its place in the row: 1140 to the one
-- player it caught (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run
-- each.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
