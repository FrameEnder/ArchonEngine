-- Copperbell Mines trash (BNpcBase 1396): a hit on its target every 20.8 s (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: 23 to players.) Its
-- timeline's `recorded_amounts` scales it to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
