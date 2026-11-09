-- Quest battle 63's ally (BNpcBase 1987, 4270049): its heal, on the player or itself when hurt, 395-412 (Connor's
-- Chronofoil capture e834e436 of 2026-11-07: five heals at 15-48 % health.)
local HEAL = 404

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(HEAL)

    return effects
end
