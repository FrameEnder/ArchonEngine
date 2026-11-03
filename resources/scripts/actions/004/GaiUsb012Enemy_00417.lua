-- Trial by Turtle's (3865) enemy (BNpcBase 1888, layout 4292977): a 3.7 s cast round itself (the Action sheet's area),
-- once, about 12 s after it came out: 192 to the player (Bob's Chronofoil captures 44df02fc and c9145fdc, 2026-11-03,
-- `questkit chronofoil --battle 0.17` and `0.11`: a miss, then a hit). Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
