-- The Praetorium's enemies' BNpcBase 14566's 29186: no cast, hit three at once: 628 to a player, 526 to a member
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18,
-- five runs.) Its area is the Action sheet's; the enemies' timelines' `recorded_amounts` scale it (the potency only
-- has to make some damage).
-- Its damage is blunt (retail's damage byte 0x73, on its hits in the five Praetorium runs but parried ones; it was magic until 2026-11-24.)

local POTENCY = 200
-- Its knockback: the Knockback sheet's row (retail's `1F .. 3`, after the damage, on every hit.)
local KNOCKBACK = 3

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
