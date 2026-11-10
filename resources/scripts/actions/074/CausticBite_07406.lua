-- Caustic Bite (Lv. 64): 150 and poison, 20 over 45 s; 35 % Hawk's Eye.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 150, TRAITS_ARC_ACTION_DAMAGE)))
    effects:damage_over_time(1200, 45.0, physicalDamage(player, classPotency(player, 20, TRAITS_ARC_ACTION_DAMAGE)))
    brdHawksEye(effects, 0.35)
    brdRepertoire(effects, player)

    return effects
end
