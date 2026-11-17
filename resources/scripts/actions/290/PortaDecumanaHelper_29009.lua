-- The Porta Decumana's helpers' 29009: a 1.9 s cast from where the second boss stands with its 29006: 446 and 1062 to
-- the members it caught (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run
-- each.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
