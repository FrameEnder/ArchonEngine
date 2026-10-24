-- The Tam-Tara Deepcroft's last boss (BNpcBase 76): a 4.7 s cast at its target that hits hard and gives status 17 for
-- 20 s (Bob's Chronofoil captures of the Tam-Tara Deepcroft, 2026-10-24: 217 to players, the median of four, and the
-- status from the EffectResult.) Its timeline's `recorded_amounts` scales the hit to retail's.
local POTENCY = 100
local STATUS = 17
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
