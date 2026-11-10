-- Fire I V (Lv. 60): 300; only in Astral Fire; an Astral Soul from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmFirePotency(player, classPotency(player, 300, TRAITS_THM_MAIM_AND_MEND))))
    effects:set_mp_cost(blmFireCost(player, 800))
    blmUseHeart(effects, player)
    if player.level >= 100 then
        effects:gauge_add("astral_soul", 1)
    end

    return effects
end
