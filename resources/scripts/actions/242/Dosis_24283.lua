-- Dosis (Lv. 1): 300 (Kardion's 170 to the partner isn't done).
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeMagic(effects, player, tierPotency(player, { { 64, 300 }, { 54, 250 }, { 1, 200 } }))

    return effects
end
