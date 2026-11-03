-- Quest 810's enemy (BNpcBase 1900, layout 4291318): a hit on its target between auto-attacks, about every 17 s:
-- 48-64 to the player (Bob's Chronofoil captures 44df02fc and c9145fdc, 2026-11-03, `questkit chronofoil --battle 0.3`
-- and `0.4`). Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
