-- An action quest battle NPCs use (Big Trouble in Little Ala Mhigo's second-wave add, BNpcBase 1981): on itself,
-- status 197 at 50 (Bob's Chronofoil captures 9faa9bac and bd51ca23, 2026-10-31: its effect, once in each; lost 20.7 s
-- later). What 197 does isn't scripted.
local STATUS = 197
local PARAM = 50
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
