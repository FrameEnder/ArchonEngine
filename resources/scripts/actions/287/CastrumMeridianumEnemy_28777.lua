-- Castrum Meridianum's first boss's soldiers' 28777 (BNpcBase 14539, 14540): a 5.7 s cast on itself: status 290
-- (param 44) on it (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17,
-- three runs.) What the status does isn't known; it's given for 30 s here.

local STATUS = 290
local STATUS_PARAM = 44
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, STATUS_PARAM, DURATION)

    return effects
end
