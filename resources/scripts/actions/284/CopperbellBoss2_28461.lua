-- The Copperbell Mines' second boss (BNpcBase 14448): a 4.7 s cast, once in three runs, 52 to a Duty Support member
-- (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
