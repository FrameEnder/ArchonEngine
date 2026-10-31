-- An action quest battle NPCs use (Nouveau Riche's lancer on your side, BNpcBase 1640; Big Trouble in Little Ala
-- Mhigo's third-wave lancers, 1730): on itself, status 65 (Bob's Chronofoil capture caa5e908, 2026-10-30: its effect,
-- once; captures 20ef0e90, 9faa9bac and bd51ca23, 2026-10-31: four times, and lost 60.5 s later where it was seen to
-- end). What 65 does isn't scripted.
local STATUS = 65
local PARAM = 0
local DURATION = 60.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
