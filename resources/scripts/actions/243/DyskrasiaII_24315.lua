-- Dyskrasia I I (Lv. 82): 170 to all enemies near.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeMagic(effects, player, 170)

    return effects
end
