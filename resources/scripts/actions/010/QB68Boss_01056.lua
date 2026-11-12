-- Parley in the Sagolii's boss's 1056 (quest battle 68, BNpcBase 1663): a 3.7 s cast on himself that gave him statuses
-- 293 and 294 (Bob's Chronofoil capture da424662 of 2026-11-12, the ActionEffect, about 20 % health). How long they last
-- wasn't seen (the battle ended 1.5 s later): 30 s is a guess. What they do isn't known.
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(293, 0, DURATION)
    effects:gain_effect_self(294, 0, DURATION)

    return effects
end
