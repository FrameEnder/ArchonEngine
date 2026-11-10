-- Double Check (Lv. 92): 180 to the first enemy, 30 % less round it.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(30)
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, levelPotency(player, 180, LEVEL_MCH_ACTION_DAMAGE)))

    return effects
end
