-- Trial by Water's water sprites (quest battle 16, BNpcBase 1681): a 2.2 s cast round itself (the Action sheet's area),
-- about 6 s after each 1120 (Bob's Chronofoil capture b23b91ef, 2026-10-26, two runs: 25.5 on you, 34 on a Wood
-- Wailer, 1-3 caught.) Its timeline's `recorded_amounts` scales the hit to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
