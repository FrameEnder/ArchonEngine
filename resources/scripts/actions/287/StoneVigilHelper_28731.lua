-- The second boss's helpers' (BNpcBase 14544) circle round the whirlwind they ride with, instant, every 0.58 s
-- (`director:sweep_bnpc`; 980 to the player). From Bob's Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with
-- Duty Support, three wipes). Its area and cast are the Action sheet's; its timeline's `recorded_amounts` scales the
-- hit to retail's.
-- Its hits gave status 2656 for 4 s and tossed whoever they hit into the air where they stood (an effect of kind 62
-- with GimmickJump row 64 and 173, then ActorControl 220 from the one hit: `effects:toss`; three hits in his captures
-- 1cefedf4 and 341808b0.)
local POTENCY = 100
local STATUS = 2656
local PARAM = 0
local DURATION = 4.0
local TOSS_GIMMICK_JUMP = 64
local TOSS_VALUE = 173

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:gain_effect(STATUS, PARAM, DURATION)
    effects:toss(TOSS_GIMMICK_JUMP, TOSS_VALUE)

    return effects
end
