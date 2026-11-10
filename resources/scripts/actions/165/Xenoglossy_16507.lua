-- Xenoglossy (Lv. 80): 890; a Polyglot.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, math.floor(classPotency(player, 890, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:gauge_add("polyglot", -1)

    return effects
end
