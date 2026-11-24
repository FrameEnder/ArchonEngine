-- The Praetorium's last boss's 28493: a 4.7 s cast on one of the party, shared with who stands by: 1317 and 1040
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18,
-- five runs.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.
-- Its damage is slashing (retail's damage byte 0x71, on its hits in the five Praetorium runs but parried ones; it was magic until 2026-11-24.)

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
