-- Bio as quest battle 61's boss uses it (BNpcBase 1746): status 179 on its target, no damage up front; retail's
-- EffectResult gave 28.8 s (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06: four uses.) Kawari's damage
-- over time for it isn't scripted.
local STATUS = 179
local DURATION = 28.8

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
