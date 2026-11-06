-- An NPC action (The Rematch's second gladiator on your side, quest battle 23, BNpcBase 666): on itself, status 3285
-- at 236 (Bob's Chronofoil capture accc33a9 of 2026-11-06: its effect, once.) How long it lasts isn't recorded (15 s
-- here); what 3285 does isn't scripted.
local STATUS = 3285
local PARAM = 236
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
