-- Grim Reaping (Lv. 80): 220 to the enemies in a cone; a Lemure Shroud for a Void Shroud.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 220)
    rprReaping(effects, player)

    return effects
end
