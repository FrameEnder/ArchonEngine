-- The Praetorium's second boss's BNpcBase 14452's 28480: no cast, on a player, with a knockback: 399 to a player, 310
-- to a member (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of
-- 2026-11-17 and 18, five runs.) The timeline's `recorded_amounts` scale it.
-- Its damage is blunt (retail's damage byte 0x73, on its hits in the five Praetorium runs but parried ones; it was magic until 2026-11-24.)

local POTENCY = 150
-- Its knockback: the Knockback sheet's row (retail's `1F .. 70`, after the damage, on every hit.)
local KNOCKBACK = 70

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
