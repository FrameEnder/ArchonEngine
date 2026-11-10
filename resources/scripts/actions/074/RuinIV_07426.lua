-- Ruin I V (Lv. 62): 520 to the target, 60 % less round it; only with Further Ruin.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 520, TRAITS_ACN_MAIM_AND_MEND)))
    effects:consume_effect(SMN_EFFECT_FURTHER_RUIN)

    return effects
end
