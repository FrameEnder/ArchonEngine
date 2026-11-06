-- Titan's Rock Buster (the Navel, BNpcBase 246): instant, on his target (the tank) and those before him (the Action
-- sheet's area): 286-380 to the players' tanks, 191 to the Duty Support Marauder (Bob's Chronofoil captures 650dbfd8, 0fd8bce0 and 48aaf6fe of 2026-11-06, the Navel). His timeline's
-- `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, physicalDamage(player, POTENCY))

    return effects
end
