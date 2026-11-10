-- Moulinet (Lv. 52): 60 to the enemies in a cone.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 60))

    return effects
end
