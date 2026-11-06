-- Stone Gaze (BNpcBase 1748 in quest battle 61): a gaze that petrifies (status 1, param 0) whoever it catches facing it
-- for 9 s, no damage (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06: 11 caught, 3 missed.) See
-- QuestBattleNpc_00513 for gazes.
local STATUS = 1
local DURATION = 9.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gaze()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
