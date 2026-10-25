-- Copperbell Mines trash (BNpcBase 1402): a 2.2 s cast every 21.2 s, hitting nobody 10 times of 14; the others gave
-- status 18 at 4 or 5, gone within about 5 s (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25; 6 s here.) What 18 does isn't scripted.
local STATUS = 18
local PARAM = 4
local DURATION = 6.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
