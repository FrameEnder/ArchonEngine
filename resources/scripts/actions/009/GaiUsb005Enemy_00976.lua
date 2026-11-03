-- Nix That's (812) enemy (BNpcBase 1887, layout 4292674): a hit on its target, once, 17-20 s after it came out: 41-85
-- to the player (Bob's Chronofoil captures 44df02fc and c9145fdc, 2026-11-03, `questkit chronofoil --battle 0.16` and
-- `0.10`). Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
