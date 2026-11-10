-- High Thunder I I (Lv. 92): 100 to the target and round it, 40 over 24 s; uses up Thunderhead.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, math.floor(classPotency(player, 100, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:damage_over_time(3872, 24.0, magicalDamage(player, math.floor(classPotency(player, 40, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:consume_effect(BLM_EFFECT_THUNDERHEAD)

    return effects
end
