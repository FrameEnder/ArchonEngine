-- Copperbell Mines trash (BNpcBase 1402): a hit every 19.2 s (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: 29 to players.) Its timeline's
-- `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
