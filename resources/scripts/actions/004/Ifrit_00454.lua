-- Ifrit's Vulcan Burst (the Bowl of Embers, BNpcBase 207): instant, 16 yalms round him, a knockback that did 1 to
-- everyone it reached (Bob's Chronofoil capture b8432613 of 2026-10-27, the Bowl of Embers with Duty Support (visit 0.16)).
-- The knockback is the Knockback sheet's row 1, after the damage, on every one of the 58 hit in 15 uses across Bob's
-- captures (kind 31, as Titan's Landslide).
local POTENCY = 100
local KNOCKBACK = 1

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
