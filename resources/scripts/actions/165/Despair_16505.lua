-- Despair (Lv. 72): 350; all the MP left (800 at least); Astral Fire III; only in Astral Fire.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmFirePotency(player, classPotency(player, 350, TRAITS_THM_MAIM_AND_MEND))))
    effects:set_mp_cost(math.max(player.mp, 800))
    blmSetElement(effects, player, 3, false)

    return effects
end
