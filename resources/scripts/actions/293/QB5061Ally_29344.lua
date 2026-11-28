-- The Ultimate Weapon's last battle's ally's 29344: no cast, on the fallen player from the middle, 36.3 s after they
-- fell. Retail's two effects, the same in Bob's Chronofoil captures 9ba7d981, 6320ae56 and 50e72631: kind 50
-- (`32 00 00 00 00 00 01 00`, meaning unknown) and status 3123 (`0E 96 64 19 00 00 33 0C`: parameter 25, and 150 and
-- 100 in the two bytes before it), which lasted to the end. 3123 doubles their maximum HP (PlayerStats with it: 2377
-- to 4754, `modifiers::MAX_HP`). They get up 13.5 s later, the director's (content/questbattle/5061.lua), and
-- status 2056 comes then.
local STATUS = 3123
local STATUS_PARAM = 25
local STATUS_BYTES = { 150, 100 }
local DURATION = 0.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:unknown_effect(50, 1)
    effects:gain_effect(STATUS, STATUS_PARAM, DURATION, STATUS_BYTES[1], STATUS_BYTES[2])

    return effects
end
