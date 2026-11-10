-- Flare Star (Lv. 100): 500 to the target, 65 % less round it; six Astral Soul, spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(65)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmFirePotency(player, classPotency(player, 500, TRAITS_THM_MAIM_AND_MEND))))
    effects:gauge_set("astral_soul", 0)

    return effects
end
