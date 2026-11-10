-- Thunder I V (Lv. 64): 80 to the target and round it, 35 over 21 s; uses up Thunderhead.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, math.floor(classPotency(player, 80, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:damage_over_time(1210, 21.0, magicalDamage(player, math.floor(classPotency(player, 35, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:consume_effect(BLM_EFFECT_THUNDERHEAD)

    return effects
end
