-- Quest 810's enemy (BNpcBase 1900, layout 4291318): a 2.2 s cast round itself (the Action sheet's area), about 11-12 s
-- after it came out: 73-92 to the player and status 9 for 12 s (Bob's Chronofoil captures 44df02fc and c9145fdc,
-- 2026-11-03, `questkit chronofoil --battle 0.3` and `0.4`: the damage, and the status' time from the EffectResult).
-- What 9 does isn't scripted. Its timeline's `recorded_amounts` scales the hit.
local POTENCY = 100
local STATUS = 9
local DURATION = 12.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
