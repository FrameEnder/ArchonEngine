-- An action of the old pugilist's kit that quest battle NPCs still use (On Holy Ground's pugilist, quest battle 22,
-- BNpcBase 640, and The Rematch's pugilists, quest battle 23, base 664): on itself, status 100 at 30 (Bob's Chronofoil
-- capture accc33a9 of 2026-11-06: its effect, every use.) How long it lasts isn't recorded (15 s here); what 100 does
-- isn't scripted.
local STATUS = 100
local PARAM = 30
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
