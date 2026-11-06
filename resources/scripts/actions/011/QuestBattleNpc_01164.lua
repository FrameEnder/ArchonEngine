-- Spirit Dart (BNpcBase 1747, quest battle 61): a cast that gives its target status 64 for 15 s, no damage (Bob's
-- Chronofoil captures 130088ec and 3c56e09f of 2026-11-06.)
local STATUS = 64
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
