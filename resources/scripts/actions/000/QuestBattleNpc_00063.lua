-- An action of the old pugilist's kit that quest battle NPCs still use (Brotherly Love's Yda, BNpcBase 1586): on
-- herself, status 103 at 5 (Bob's Chronofoil capture 5fcc970c, 2026-10-30: its effect). How long it lasts wasn't
-- seen: 15 s is a guess. What 103 does isn't scripted.
local STATUS = 103
local PARAM = 5
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
