-- Refulgent Arrow (Lv. 70): 280; only with Hawk's Eye or Barrage (it uses them up).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 280, TRAITS_ARC_ACTION_DAMAGE)))
    brdUseHawksEye(effects, player)
    brdRepertoire(effects, player)

    return effects
end
