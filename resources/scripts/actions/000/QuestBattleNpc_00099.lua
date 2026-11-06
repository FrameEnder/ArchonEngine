-- An action of the old archer's kit that quest NPCs still use (The Rematch's archers in Southern Thanalan, BNpcBase
-- 1242): on itself, status 123 at 10 (Bob's Chronofoil capture accc33a9 of 2026-11-06: its effect, each one's once,
-- as it was pulled.) How long it lasts isn't recorded (20 s here); what 123 does isn't scripted.
local STATUS = 123
local PARAM = 10
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
