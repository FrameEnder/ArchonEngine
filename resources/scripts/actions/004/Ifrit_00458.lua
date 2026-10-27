-- Ifrit's Hellfire (the Bowl of Embers, BNpcBase 207): a 1.7 s cast from the arena's middle once the Infernal Nail
-- falls, 30 yalms round him: 230 to the player, 166-183 to the Lancer and Thaumaturge, 74 to the Marauder (Bob's Chronofoil capture b8432613 of 2026-10-27, the Bowl of Embers with Duty Support (visit 0.16)).
-- His timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
