-- The Copperbell Mines' second ambushing gigas (BNpcBase 1403): once a fight, on itself, status 203 at 150 (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25:
-- its effect, and up to 29 s left in the StatusEffectLists, so 30 s.) What 203 does isn't scripted.
local STATUS = 203
local PARAM = 150
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
