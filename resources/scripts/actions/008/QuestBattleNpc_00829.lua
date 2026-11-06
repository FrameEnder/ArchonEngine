-- An NPC action (On Holy Ground's pugilist, quest battle 22, BNpcBase 640; a 5 s cast): on itself, status 291 at 176
-- (Bob's Chronofoil capture accc33a9 of 2026-11-06: its effect, once, at 9 % health near the end.) How long it lasts
-- isn't recorded (15 s here); what 291 does isn't scripted.
local STATUS = 291
local PARAM = 176
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
