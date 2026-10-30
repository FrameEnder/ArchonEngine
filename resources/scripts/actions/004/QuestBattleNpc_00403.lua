-- An action quest battle and quest enemies use (Nouveau Riche's gladiator, BNpcBase 1645; Presence of the Enemy's
-- fourth ambusher, 1898): on itself, status 61 at 50 (Bob's Chronofoil captures 5fcc970c and caa5e908, 2026-10-30:
-- its effects; 21.2 s between two uses). 20 s is a guess. What 61 does isn't scripted.
local STATUS = 61
local PARAM = 50
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
