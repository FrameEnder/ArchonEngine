-- An action quest battle NPCs use (Nouveau Riche's lancer on your side, BNpcBase 1640): on itself, status 65 (Bob's
-- Chronofoil capture caa5e908, 2026-10-30: its effect, once). How long it lasts wasn't seen: 20 s is a guess. What 65
-- does isn't scripted.
local STATUS = 65
local PARAM = 0
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
