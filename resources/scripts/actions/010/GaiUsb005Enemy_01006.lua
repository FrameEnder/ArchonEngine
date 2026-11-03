-- Nix That's (812) enemy (BNpcBase 1887, layout 4292674): a 3.7 s cast round itself (the Action sheet's area), once,
-- about 10-11 s after it came out: 160 to the player (Bob's Chronofoil captures 44df02fc and c9145fdc, 2026-11-03,
-- `questkit chronofoil --battle 0.16` and `0.10`: dodged in the first). Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
