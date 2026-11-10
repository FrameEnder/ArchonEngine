-- Dream Within A Dream (Lv. 56): three hits of 180.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 180))
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 180))
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 180))

    return effects
end
