-- Doton (Lv. 45): a patch of earth, 80 a second for 18 s (here its damage over time on the target) (three mudras ending in Chi).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "doton")

    return effects
end
