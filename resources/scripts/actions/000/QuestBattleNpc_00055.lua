-- An action of the old pugilist's kit that quest battle NPCs still use (Into a Copper Hell's pugilist, BNpcBase 960):
-- on itself, status 99 at 12 (Bob's Chronofoil capture of 2026-10-25: its effect, and up to 14.4 s left in the
-- StatusEffectLists, so 15 s.) What 99 does isn't scripted.
local STATUS = 99
local PARAM = 12
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, PARAM, DURATION)

    return effects
end
