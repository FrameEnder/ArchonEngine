-- An action quest battle NPCs use on themselves (the conjurers in quest battles 60 and 61 (BNpcBases 1739, 1751,
-- 1732)): status 294 for 15 s on itself (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06: the
-- EffectResult.) Not verified for players who use it.
local STATUS = 294
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, 0, DURATION)

    return effects
end
