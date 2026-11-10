-- Crimson Strike (Lv. 86): 560 to the target, 60 % less round it; only with Crimson Strike Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 560, TRAITS_ACN_MAIM_AND_MEND)))
    effects:consume_effect(SMN_EFFECT_CRIMSON_STRIKE_READY)

    return effects
end
