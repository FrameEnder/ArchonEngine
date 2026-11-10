-- Hyosho Ranryu (Lv. 76): 1,300 (Hyoton under Kassatsu).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "hyosho")

    return effects
end
