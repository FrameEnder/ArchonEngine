-- Brotherly Love's beast (quest battle 57, BNpcBase 1585): a 2.7 s cast on itself catching Yda and Papalymo (Bob's
-- Chronofoil capture 5fcc970c, 2026-10-30). Its effect on each was kind 48 (63 on Yda, 52 on Papalymo; unknown to
-- Kawari) and, the first time, both were left at exactly 80 % as it landed (its EffectResults); the second, 30.7 s
-- later, took nothing. Here it's a hit, scaled by its timeline's `recorded_amounts` to 20 % of their 25890, and the
-- director casts it once.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
