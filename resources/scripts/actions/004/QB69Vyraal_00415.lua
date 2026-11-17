-- Quest battle 69's vyraals' second action (BNpcBase 2228: a 2.5 s cast, a line of 4 yalms, every 12 s): 192 on the
-- second ally the once it hit, against its auto-attack's 61 to 87 (Bob's Chronofoil capture 36c7d119 of 2026-11-17,
-- five uses). Its timeline's `recorded_amounts` scales it.
local POTENCY = 200

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
