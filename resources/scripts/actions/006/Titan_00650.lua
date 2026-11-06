-- Titan's Landslide (the Navel, BNpcBase 246): a 2.7 s cast, a line before him (the Action sheet's area): 602-659 to
-- the players it caught, and it knocks them back (the Knockback sheet's row 37, after the damage, every hit), as
-- Bob's Chronofoil captures 650dbfd8, 0fd8bce0 and 48aaf6fe of 2026-11-06, the Navel show it. His timeline's `recorded_amounts` scale it.
local POTENCY = 100
local KNOCKBACK = 37

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
