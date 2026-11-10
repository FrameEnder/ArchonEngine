-- Lemures Scythe (Lv. 86): 100 to the enemies in a cone; two Void Shroud.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 100)
    effects:gauge_add("void", -2)

    return effects
end
