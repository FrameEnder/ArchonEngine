-- The Copperbell Mines' gigas (BNpcBase 1400): a 2.7 s cast, once in three runs, 38 to a Duty Support member with
-- effect 31 (16), which isn't scripted (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
