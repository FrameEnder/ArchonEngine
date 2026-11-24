-- The Praetorium's last boss's fight, BNpcBase 14455's: a 4.7 s cast on one of the party each: 671 to a player, 546 to a
-- member (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17
-- and 18, five runs.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.
-- Its damage is piercing (retail's damage byte 0x72, on its hits in the five Praetorium runs but parried ones; it was magic until 2026-11-24.)

local POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))

    return effects
end
