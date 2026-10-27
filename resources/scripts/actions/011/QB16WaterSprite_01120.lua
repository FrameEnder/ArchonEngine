-- Trial by Water's water sprites (quest battle 16, BNpcBase 1681): a 0.7 s cast on you about every 13 s (Bob's
-- Chronofoil capture b23b91ef, 2026-10-26, two runs: 26, the median of 11.) Its timeline's `recorded_amounts` scales
-- the hit to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
