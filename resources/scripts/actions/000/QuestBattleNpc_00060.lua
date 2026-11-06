-- An action of the old pugilist's kit that quest battle NPCs still use (On Holy Ground's pugilist, quest battle 22,
-- BNpcBase 640, and The Rematch's, quest battle 23, bases 660 and 664): on itself, status 104 at 246 (Bob's Chronofoil
-- capture accc33a9 of 2026-11-06: its effect, every use, once each early in the fight: a stance.) How long it lasts
-- isn't recorded (it was used once a fight: 600 s here); what 104 does isn't scripted.
local STATUS = 104
local PARAM = 246
local DURATION = 600.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
