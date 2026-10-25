-- The Copperbell Mines' last boss (BNpcBase 14537): a 3.7 s cast every 47.8 s, 125 to a player (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25.) Its
-- timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
