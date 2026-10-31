-- An action quest battle NPCs use (Big Trouble in Little Ala Mhigo's pugilists and lancers, BNpcBases 1978 and 1980):
-- on itself, status 204 at 50 (Bob's Chronofoil capture 20ef0e90, 2026-10-31: its effect, three times; lost 30.5 s
-- later). What 204 does isn't scripted.
local STATUS = 204
local PARAM = 50
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
