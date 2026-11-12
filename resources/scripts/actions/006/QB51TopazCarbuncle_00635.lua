-- Curl, the Topaz Carbuncle's in Over the Rails (quest battle 51, BNpcBase 2056): on itself, status 310 (param 236; Bob's
-- Kawari Recorder run of 2026-11-12, once.) How long it lasts wasn't seen (one sample with it): 10 s here; what 310 does
-- isn't scripted.
local STATUS = 310
local PARAM = 236
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
