-- High Fire I I (Lv. 82): 100 to the target and round it; Astral Fire III, Umbral Ice gone.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmFirePotency(player, classPotency(player, 100, TRAITS_THM_MAIM_AND_MEND))))
    effects:set_mp_cost(blmFireCost(player, 1500))
    blmUseHeart(effects, player)
    blmSetElement(effects, player, 3, false)

    return effects
end
