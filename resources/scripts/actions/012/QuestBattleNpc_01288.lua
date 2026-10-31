-- An action quest battle NPCs use (Big Trouble in Little Ala Mhigo's boss, BNpcBase 1977): on someone of the other side,
-- status 15 at 226, no damage (Bob's Chronofoil capture 20ef0e90, 2026-10-31: four uses, the player once; lost at most
-- 16.1 s later). What 15 does isn't scripted.
local STATUS = 15
local PARAM = 226
local DURATION = 16.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
