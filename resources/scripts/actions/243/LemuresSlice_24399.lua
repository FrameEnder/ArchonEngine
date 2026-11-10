-- Lemures Slice (Lv. 86): 280; two Void Shroud.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 280)
    effects:gauge_add("void", -2)

    return effects
end
