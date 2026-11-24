-- The Jade Stoa's Byakko (10208): a 3.7 s cast on one of the party: stuns them (201, until thrown). From Bob's
-- Chronofoil capture 84700114 (2026-11-24, the Jade Stoa unsynced).

local STUN = 201

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STUN, 0, 0.0)
    return effects
end
