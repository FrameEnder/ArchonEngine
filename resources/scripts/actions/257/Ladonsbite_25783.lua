-- Ladonsbite (Lv. 82): 140 to the enemies in a cone; 35 % Hawk's Eye.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 140, TRAITS_ARC_ACTION_DAMAGE)))
    brdHawksEye(effects, 0.35)

    return effects
end
