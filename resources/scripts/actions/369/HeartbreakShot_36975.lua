-- Heartbreak Shot (Lv. 92): 180.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 180, TRAITS_ARC_ACTION_DAMAGE)))

    return effects
end
