-- Seeing Eye to Winged Eye's (786) quest enemy (BNpcBase 1899, layout 4285321): on the player, status 18 at 16, no
-- damage, twice 20.9 s apart (Bob's Chronofoil capture 412ba91d, 2026-11-01: lost 15.3 s after both). What 18 does
-- isn't scripted.
local STATUS = 18
local PARAM = 16
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
