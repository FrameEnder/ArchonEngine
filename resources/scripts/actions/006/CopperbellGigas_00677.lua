-- The Copperbell Mines' gigas (BNpcBase 1400): a 2.7 s cast, once in three runs, 38 to a Duty Support member and a
-- knockback, the Knockback sheet's row 16, after it (effect 31; Bob's Chronofoil captures of the Copperbell Mines,
-- 2026-10-25.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100
local KNOCKBACK = 16

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
