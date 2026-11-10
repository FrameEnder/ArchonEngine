-- Rhizomata (Lv. 74): an Addersgall.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_add("addersgall", 1)

    return effects
end
