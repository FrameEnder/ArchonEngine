-- Tam-Tara Deepcroft trash (BNpcBase 1388): a 2.2-2.7 s cast every 10.6 s (Bob's Chronofoil captures of 2026-10-24:
-- 136 to players, the median of the three that landed on one.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
