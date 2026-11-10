-- Foul (Lv. 70): 600 to the target, 25 % less round it; a Polyglot.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(25)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, math.floor(classPotency(player, 600, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:gauge_add("polyglot", -1)

    return effects
end
