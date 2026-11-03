-- Like Mother, Like Daughter's wind sprite on your side (quest battle 56, BNpcBase 1691): a 3.7 s cast on an enemy
-- every 12.6 s that gave it status 17 with param 50 and no damage (Bob's Chronofoil capture b23b91ef, 2026-10-26, its
-- ActionEffects). The Lominsan Way's quest battle (59) has it too: its first enemy (BNpcBase 1769) casts it on the
-- player once, 21.5 s into the fight, status 17 with param 50 for 30 s (Bob's Chronofoil captures 44df02fc and
-- c9145fdc, 2026-11-03: the EffectResult, every run). What 17 does isn't scripted.
local STATUS = 17
local PARAM = 50
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
