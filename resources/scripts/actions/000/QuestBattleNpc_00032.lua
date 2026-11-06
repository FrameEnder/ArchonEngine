-- An action quest battle NPCs use on themselves (quest battle 60's marauder (BNpcBase 1731)): status 83 for 20 s on
-- itself (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06: the EffectResult.) Not verified for players
-- who use it.
local STATUS = 83
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, 0, DURATION)

    return effects
end
