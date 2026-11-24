-- The Praetorium's second boss's 28476: a 4.7 s cast on everyone, with a knockback: 620 to a player, 572 to a member
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18,
-- five runs.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 250
-- Its knockback: the Knockback sheet's row (retail's `1F .. 156`, after the damage, on every hit.)
local KNOCKBACK = 156

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
