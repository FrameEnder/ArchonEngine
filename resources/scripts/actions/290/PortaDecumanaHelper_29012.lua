-- The Porta Decumana's helpers' 29012: a 4.7 s cast on each of the party where they stand, with the second boss's
-- 29011: 925 to 1001 to a player, 369 to 958 to a member (Connor's Chronofoil capture af05916c of 2026-11-09 and
-- Bob's 9ba7d981 of 2026-11-17, one run each.) Its area is the Action sheet's; the timeline's `recorded_amounts`
-- scale it.

local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
