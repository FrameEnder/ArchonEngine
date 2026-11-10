-- High Thunder (Lv. 92): 150, 60 over 30 s; uses up Thunderhead.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, math.floor(classPotency(player, 150, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:damage_over_time(3871, 30.0, magicalDamage(player, math.floor(classPotency(player, 60, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:consume_effect(BLM_EFFECT_THUNDERHEAD)

    return effects
end
