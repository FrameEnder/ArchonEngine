-- Katon (Lv. 35): 350 to the target and round it (two mudras ending in Ten).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "katon")

    return effects
end
