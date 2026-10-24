-- The Tam-Tara Deepcroft's room bosses (BNpcBase 567): a hit on their target every 9.7 s (Bob's Chronofoil captures of
-- the Tam-Tara Deepcroft, 2026-10-24: 54 to players.) Its timeline's `recorded_amounts` scales it to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
