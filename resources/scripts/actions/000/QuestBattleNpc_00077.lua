-- An action of the old lancer's kit that quest battle NPCs still use (Nouveau Riche's boss, BNpcBase 1639): on itself,
-- status 114 at 40 (Bob's Chronofoil capture caa5e908, 2026-10-30: its effect, once). How long it lasts wasn't seen:
-- 20 s is a guess. What 114 does isn't scripted.
local STATUS = 114
local PARAM = 40
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
