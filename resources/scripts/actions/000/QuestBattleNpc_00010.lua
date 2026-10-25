-- An action of the old gladiator's kit that quest battle NPCs still use (Into a Copper Hell's gladiator, BNpcBase 961):
-- on itself, status 71 at 246 (Bob's Chronofoil capture of 2026-10-25: its effect, and up to 17.9 s left in the
-- StatusEffectLists, so 20 s.) What 71 does isn't scripted.
local STATUS = 71
local PARAM = 246
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
