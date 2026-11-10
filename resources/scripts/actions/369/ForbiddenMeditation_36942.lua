-- Forbidden Meditation (Lv. 54): opens a chakra (five at most; all five out of combat isn't told apart here).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_add("chakra", 1)

    return effects
end
