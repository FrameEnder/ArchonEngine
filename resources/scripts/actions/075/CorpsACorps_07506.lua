-- Corps A Corps (Lv. 6): 130 and a rush to the target (movement isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 130))

    return effects
end
