-- A battle NPC's self-buff: status 63 on itself for 15 s after a 2.2 s cast (retail's ActionEffect on itself:
-- `0e e2 00 0000 00 3f00`, and the EffectResult's status 63 with parameter 0 for 15.0 s; Return to Your Dust's enemy,
-- BNpcBase 2195, twice and Looking for Trouble's 328 once, Bob's Chronofoil capture 651981e7 of 2026-11-16.) What the
-- status does isn't scripted.
local STATUS = 63
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, 0, DURATION)

    return effects
end
