-- Quest battle 63's boss (BNpcBase 1988, 4270036): a 2.7 s cast on itself under 40 % health, status 61 (Connor's
-- Chronofoil capture e834e436 of 2026-11-07: the EffectResult; it fell 20 s later with the status still on, so the
-- duration is a guess.)
local STATUS = 61
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, 0, DURATION)

    return effects
end
