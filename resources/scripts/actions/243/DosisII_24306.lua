-- Dosis I I (Lv. 72): 320 (Kardion's 170 to the partner isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeMagic(effects, player, 320)

    return effects
end
