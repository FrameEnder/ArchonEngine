-- The Jade Stoa's Byakko (10209): no cast: seizes the one he stunned (1477, until thrown), 1.1 s after. From Bob's
-- Chronofoil capture 84700114 (2026-11-24, the Jade Stoa unsynced).

local SEIZED = 1477

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SEIZED, 0, 0.0)
    return effects
end
