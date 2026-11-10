-- Blizzard I V (Lv. 58): 300; three Umbral Hearts; only in Umbral Ice (free there).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmIcePotency(player, classPotency(player, 300, TRAITS_THM_MAIM_AND_MEND))))
    effects:set_mp_cost(blmIceCost(player, 800))
    effects:gauge_set("umbral_hearts", 3)
    blmSetElement(effects, player, blmElement(player), true)

    return effects
end
