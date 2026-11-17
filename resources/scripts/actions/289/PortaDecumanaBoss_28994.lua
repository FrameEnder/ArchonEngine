-- The Porta Decumana's first boss's 28994: no cast, on itself, 0.2 s after its long cast is stopped: status 3010 off
-- it (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.)

local STATUS = 3010

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:lose_effect(STATUS)

    return effects
end
