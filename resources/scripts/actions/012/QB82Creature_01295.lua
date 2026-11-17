-- Quest battle 82's creature's action (BNpcBase 2219: instant, every 16.8 s): 328 to 338 on the player against its
-- auto-attack's 188 (Bob's Chronofoil capture a75e1390 of 2026-11-17, four uses). Its timeline's `recorded_amounts`
-- scales it.
local POTENCY = 200

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
