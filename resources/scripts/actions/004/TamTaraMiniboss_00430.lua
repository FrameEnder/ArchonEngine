-- The Tam-Tara Deepcroft's room bosses (BNpcBase 567): a 1.2 s cast on themselves as each fight starts and every 33 s,
-- giving them status 208 at 100 for 30 s (Bob's Chronofoil captures of 2026-10-24: its effect and StatusEffectList.)
-- What 208 does isn't scripted.
local STATUS = 208
local PARAM = 100
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
