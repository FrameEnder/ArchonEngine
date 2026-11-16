-- A battle NPC's self-buff: status 207 with parameter 100 on itself for 30 s after a 1.2 s cast (retail's ActionEffect
-- on itself: `0e 32 04 6400 00 cf00`, and the EffectResult's status 207 with 100 for 30.0 s; Looking for Trouble's
-- BNpcBase 329, Bob's Chronofoil capture 651981e7 of 2026-11-16; the same result seven times in the capture, the
-- others open-world NPCs'.) What the status does isn't scripted.
local STATUS = 207
local PARAM = 100
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
