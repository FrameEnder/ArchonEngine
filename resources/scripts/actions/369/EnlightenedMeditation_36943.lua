-- Enlightened Meditation (Lv. 74): opens a chakra.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_add("chakra", 1)

    return effects
end
