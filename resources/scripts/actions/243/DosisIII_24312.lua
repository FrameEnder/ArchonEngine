-- Dosis I I I (Lv. 82): 380 (Kardion's 170 to the partner isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeMagic(effects, player, 380)

    return effects
end
