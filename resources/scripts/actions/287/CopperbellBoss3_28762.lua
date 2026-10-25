-- The Copperbell Mines' last boss (BNpcBase 14537): a 5.7 s cast, 125 to the players it caught (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25.) Its
-- timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
