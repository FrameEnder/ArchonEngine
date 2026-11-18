-- Cape Westwind's boss's 28866: a 4.7 s cast on himself that gives him two statuses for 7 s, in its own ActionEffect
-- (`0E 02 00 01 00 00 A8 02`, `0E 00 00 8A 00 00 93 08`): 680 with param 1 and 2195 with param 394. The second's
-- param is what shows: his raised shield. They are on him 0.8 s after it lands (EffectResult: 7.0 s each, himself
-- their source) and come off with their params 7.0 s later (Bob's Chronofoil capture 178b0de0 of 2026-11-18; the
-- same in 667a68c5.) Until 2026-11-18 the director gave both with param 0, so no shield showed (Bob: "check the
-- shield vfx, cause it was not working".) Given here as the action's statuses on its user (effect 15); retail's
-- two are effect 14 on its target, which is himself. What a hit from in front does then is the director's
-- (content/questbattle/5060.lua.)
local STANCE = 680
local STANCE_PARAM = 1
local SHIELD = 2195
local SHIELD_PARAM = 394
local DURATION = 7.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STANCE, STANCE_PARAM, DURATION)
    effects:gain_effect_self(SHIELD, SHIELD_PARAM, DURATION)

    return effects
end
