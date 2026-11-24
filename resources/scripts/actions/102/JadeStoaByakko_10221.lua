-- The Jade Stoa's Byakko (10221): a 2.7 s cast at the middle: the orbs (BNpcBase 8441) come out of it from 2.5 s
-- after it ends (1480 on him, until they stop). From Bob's Chronofoil capture 84700114 (2026-11-24, the Jade Stoa
-- unsynced).

local STATUS = 1480

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(STATUS, 0, 0.0)
    return effects
end
