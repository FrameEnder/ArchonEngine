-- Ifrit's Incinerate (the Bowl of Embers, BNpcBase 207): instant, a 10-yalm cone the way he faces, on the tank (the
-- Action sheet's area). 57 to the Duty Support Marauder, 146-179 to the player once the tank had fallen (Bob's Chronofoil capture b8432613 of 2026-10-27, the Bowl of Embers with Duty Support (visit 0.16)): his
-- timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
