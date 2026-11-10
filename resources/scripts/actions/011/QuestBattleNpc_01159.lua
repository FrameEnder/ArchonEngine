-- Eye for an Eye as quest battle 61's boss uses it (BNpcBase 1746) on one of its adds (BNpcBase 1748): status 185, no
-- damage or heal (Bob's Chronofoil captures 130088ec, 3c56e09f and 21f967d2: eleven uses in five runs, effect kind 14
-- with 185 every time.) How long it lasts wasn't recorded (20 s, a guess from the effect's fifth byte); what it does
-- isn't scripted.
local STATUS = 185
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
