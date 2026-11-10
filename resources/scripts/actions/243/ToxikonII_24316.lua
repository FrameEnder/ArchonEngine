-- Toxikon I I (Lv. 82): 380 to the first enemy, 50 % less round it; an Addersting.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeMagic(effects, player, 380, 50)
    effects:gauge_add("addersting", -1)

    return effects
end
