-- The Praetorium's second boss's 28481: a 3.2 s cast round itself (a circle of 7 yalms): 1636 to a member (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18, five
-- runs.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.
-- Its damage is blunt (retail's damage byte 0x73, on its hits in the five Praetorium runs but parried ones; it was magic until 2026-11-24.)

local POTENCY = 500
-- Its knockback: the Knockback sheet's row (retail's `1F .. 156`, after the damage, on every hit.)
local KNOCKBACK = 156

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
