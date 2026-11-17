-- Quest battle 82's master's action at the player (BNpcBase 2218: a 3.2 s cast aimed at a spot, every 28.8 s after
-- its creature is gone, the script's: content/questbattle/82.lua): 1319 and 1543 on the player the two times it
-- landed (Bob's Chronofoil capture a75e1390 of 2026-11-17, three uses). Its timeline's `recorded_amounts` scales it.
local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
