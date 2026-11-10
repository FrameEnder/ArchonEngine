-- Lord Of Crowns (Lv. 70): 400 to all enemies near.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, 400)
    effects:gauge_set("ast_arcana", 0)

    return effects
end
