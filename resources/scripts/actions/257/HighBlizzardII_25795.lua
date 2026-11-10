-- High Blizzard I I (Lv. 82): 100 to the target and round it; Umbral Ice III, Astral Fire gone.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmIcePotency(player, classPotency(player, 100, TRAITS_THM_MAIM_AND_MEND))))
    effects:set_mp_cost(blmIceCost(player, 800))
    blmSetElement(effects, player, -3, true)

    return effects
end
