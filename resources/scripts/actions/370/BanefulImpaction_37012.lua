-- Baneful Impaction (Lv. 92): 140 over 15 s to the target and round it; only with Impact Imminent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(SCH_EFFECT_BANEFUL_IMPACTION, 15.0, magicalDamage(player, classPotency(player, 140, TRAITS_ACN_MAIM_AND_MEND)))
    effects:consume_effect(SCH_EFFECT_IMPACT_IMMINENT)

    return effects
end
