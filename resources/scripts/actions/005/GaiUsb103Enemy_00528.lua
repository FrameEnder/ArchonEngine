-- When the Worm Turns' (822) second enemy (BNpcBase 1890, layout 4293190): a 2.2 s cast in front of itself (the Action
-- sheet's area), about every 11 s: 79-105 to the player and status 15 for 15 s (Bob's Chronofoil captures 44df02fc and
-- c9145fdc, 2026-11-03, `questkit chronofoil --battle 0.18` and `0.12`: the damage, and the status' time from the
-- EffectResult). What 15 does isn't scripted. Its timeline's `recorded_amounts` scales the hit.
local POTENCY = 100
local STATUS = 15
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BREATH, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
