-- Iron Jaws (Lv. 56): 100; Caustic Bite and Stormbite on the target back to their full time; 35 % Hawk's Eye.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 100, TRAITS_ARC_ACTION_DAMAGE)))
    local caustic = player.level >= 64
    effects:damage_over_time(caustic and 1200 or 124, 45.0, physicalDamage(player, classPotency(player, caustic and 20 or 15, TRAITS_ARC_ACTION_DAMAGE)))
    effects:damage_over_time(caustic and 1201 or 129, 45.0, physicalDamage(player, classPotency(player, caustic and 25 or 20, TRAITS_ARC_ACTION_DAMAGE)))
    brdHawksEye(effects, 0.35)
    brdRepertoire(effects, player)

    return effects
end
