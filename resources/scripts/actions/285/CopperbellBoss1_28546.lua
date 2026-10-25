-- The Copperbell Mines' first boss (BNpcBase 14460): a 3.7 s cast every 37 s (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: 43-46 to Duty Support members,
-- no player caught.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
