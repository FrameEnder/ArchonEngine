-- Mountain Buster (Lv. 86): 160 to the target, 60 % less round it; only with Titan's Favor.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 160, TRAITS_ACN_MAIM_AND_MEND)))
    effects:consume_effect(SMN_EFFECT_TITANS_FAVOR)

    return effects
end
