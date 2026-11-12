-- An action the green lavers of Poisoned Hearts (quest 1058, BNpcBase 1924) use on their side: status 151 on the one
-- it's cast on (2.2 s cast, five times on the mud pie princesses beside them; Bob's Chronofoil capture da424662 of
-- 2026-11-12, the ActionEffect), as quest battle 62's conjurer's 129. Its length wasn't seen to end: 30 s is a guess.
local STATUS = 151
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
