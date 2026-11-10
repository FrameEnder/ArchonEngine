-- Suiton (Lv. 45): 580; Shadow Walker (20 s) (three mudras ending in Jin).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "suiton")

    return effects
end
