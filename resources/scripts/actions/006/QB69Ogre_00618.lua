-- Quest battle 69's first part's big enemy's action (BNpcBase 1559: a 2.2 s cast, an area around it, every 10.5 s):
-- 652 to 1669 on the allies (median 1574) against its auto-attack's 196 to 374; it never hit the player (Bob's
-- Chronofoil captures 36c7d119 and b98a60bc of 2026-11-17, thirteen uses, two misses). Its timeline's
-- `recorded_amounts` scales it.
local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
