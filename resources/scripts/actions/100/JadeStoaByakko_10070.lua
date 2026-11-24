-- The Jade Stoa's Byakko (10070): no cast: stuns one of the party (201, until thrown) for his second seizing of a
-- block. From Bob's Chronofoil capture 84700114 (2026-11-24, the Jade Stoa unsynced).

local STUN = 201

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STUN, 0, 0.0)
    return effects
end
