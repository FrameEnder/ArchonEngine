-- Brotherly Love's beast (quest battle 57, BNpcBase 1585): a 2.2 s cast every 30.7 s that hit Papalymo for 102 (Bob's
-- Chronofoil capture 5fcc970c, 2026-10-30; the other one missed). Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
