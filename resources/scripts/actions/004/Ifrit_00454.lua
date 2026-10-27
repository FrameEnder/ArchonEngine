-- Ifrit's Vulcan Burst (the Bowl of Embers, BNpcBase 207): instant, 16 yalms round him, a knockback that did 1 to
-- everyone it reached (Bob's Chronofoil capture b8432613 of 2026-10-27, the Bowl of Embers with Duty Support (visit 0.16)). The knockback's effect isn't decoded (as the Copperbell Mines' 28462), so only the 1.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
