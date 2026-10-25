-- Copperbell Mines trash (BNpcBase 1421): a 2.2 s cast on itself, giving it status 53 at 50 (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: its effect, and
-- 13.7-13.9 s left in the StatusEffectLists seen, so 15 s here). What 53 does isn't scripted.
local STATUS = 53
local PARAM = 50
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
