-- Radiant Shield, Ifrit-Egi's in Austerities of Flame (quest battle 79, BNpcBase 1513): on itself, status 313 (param
-- 100) for 30 s, as it invoked the crimson nails (Bob's Kawari Recorder run of 2026-11-12: gained 0.8 s after, lost
-- 30.7 s later.) What 313 does isn't scripted: its health went on falling under it.
local STATUS = 313
local PARAM = 100
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
