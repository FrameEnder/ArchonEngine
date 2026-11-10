-- Gauss Round (Lv. 15): 130.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, levelPotency(player, 130, LEVEL_MCH_ACTION_DAMAGE)))

    return effects
end
