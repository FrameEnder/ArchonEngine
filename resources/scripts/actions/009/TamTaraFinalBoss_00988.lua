-- The Tam-Tara Deepcroft's last boss (BNpcBase 76): a hit that drains a little back (Bob's Chronofoil captures of the
-- Tam-Tara Deepcroft, 2026-10-24: 38 to players, and 5-10 back to the boss.) Its timeline's `recorded_amounts` scales
-- the hit; against players a drain gives back what the hit took (see action.rs), more than retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local damage = magicalDamage(player, POTENCY)
    effects:damage(DAMAGE_TYPE_MAGIC, damage)
    effects:heal_self(math.floor(damage / 4))

    return effects
end
