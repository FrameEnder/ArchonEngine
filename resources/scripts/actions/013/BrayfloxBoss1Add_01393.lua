-- An action of Brayflox's Longstop's first boss's add 425 (BNpcBase 425) (territory 1041): a breath in front of it,
-- no hit: status 18 (a poison) for 30 s on those it reaches (11 times; its tick 16 on the Duty Support tank, the
-- timeline's recorded amount). From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run with Duty
-- Support). Its area and cast are the Action sheet's.
local STATUS = 18
local DURATION = 30.0
local TICK_POTENCY = 30

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(STATUS, DURATION, magicalDamage(player, TICK_POTENCY))

    return effects
end
