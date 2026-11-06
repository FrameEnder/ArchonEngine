-- Virus as quest battle 61's boss uses it (BNpcBase 1746): status 182 for 10 s on its target, no damage (Bob's
-- Chronofoil captures 130088ec and 3c56e09f of 2026-11-06.)
local STATUS = 182
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
