-- An NPC action (On Holy Ground's three in Western Thanalan, BNpcBase 1243; a 2.5 s cast): on itself, status 8 at 226
-- (Bob's Chronofoil capture accc33a9 of 2026-11-06: its effect, each of the three's once.) How long it lasts isn't
-- recorded (15 s here); what 8 does isn't scripted.
local STATUS = 8
local PARAM = 226
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
