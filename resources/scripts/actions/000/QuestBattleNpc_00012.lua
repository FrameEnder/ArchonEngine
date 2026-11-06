-- An action quest battle NPCs use on themselves (the gladiators in quest battles 60 and 61 and the boss of 60
-- (BNpcBases 1738, 1750, 1735)): status 72 for 20 s on itself (Bob's Chronofoil captures 130088ec and 3c56e09f of
-- 2026-11-06: the EffectResult.) Not verified for players who use it.
local STATUS = 72
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, 0, DURATION)

    return effects
end
