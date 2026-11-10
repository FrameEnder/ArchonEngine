-- Huton (Lv. 45): 240 to the target and round it; Shadow Walker (20 s) (three mudras ending in Ten).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "huton")

    return effects
end
