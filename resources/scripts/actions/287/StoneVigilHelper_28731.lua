-- The second boss's helpers' (BNpcBase 14544) circle round the whirlwind they ride with, instant, every 0.58 s
-- (`director:sweep_bnpc`; 980 to the player). From Bob's Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with
-- Duty Support, three wipes). Its area and cast are the Action sheet's; its timeline's `recorded_amounts` scales the
-- hit to retail's.
-- Its hits gave status 2656 for 4 s (and an effect of kind 62, 173, not sent here).
local POTENCY = 100
local STATUS = 2656
local PARAM = 0
local DURATION = 4.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
