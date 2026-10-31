-- He Ain't Heavy's (775) quest enemy (BNpcBase 1909, layout 4278416): a 2.2 s cast placed where it aims, every 9.9 s,
-- which hit the player for 130 once and missed once (Bob's Chronofoil capture a0149e5e, 2026-10-31.) Its timeline's
-- `recorded_amounts` scales it to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
