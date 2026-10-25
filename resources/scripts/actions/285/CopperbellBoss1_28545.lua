-- The Copperbell Mines' first boss (BNpcBase 14460): a 4.7 s cast at its target every 81 s (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: 118 to players,
-- 60 to the Duty Support tank.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
