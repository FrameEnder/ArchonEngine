-- Copperbell Mines trash (BNpcBase 1396): a 2.2 s cast on itself giving it status 8 at 226 (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: its effect, and up
-- to 30.0 s left in the StatusEffectLists.) What 8 does isn't scripted.
local STATUS = 8
local PARAM = 226
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
