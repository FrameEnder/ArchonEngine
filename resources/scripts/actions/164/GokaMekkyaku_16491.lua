-- Goka Mekkyaku (Lv. 76): 850 to the target and round it (Katon under Kassatsu).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "goka")

    return effects
end
