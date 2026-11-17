-- Quest battle 69's vyraals' first action (BNpcBase 2228: a 4.0 s cast, an area around it, once each): 787 to 1156
-- on the two allies (median 971.5) against its auto-attack's 61 to 87, and status 9 on them, which isn't given (how
-- long it lasts wasn't read; Bob's Chronofoil capture 36c7d119 of 2026-11-17, two uses). Its timeline's
-- `recorded_amounts` scales it.
local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
