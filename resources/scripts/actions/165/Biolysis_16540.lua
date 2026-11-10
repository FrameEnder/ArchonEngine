-- Biolysis (Lv. 72): 85 over 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(SCH_EFFECT_BIOLYSIS, 30.0, magicalDamage(player, classPotency(player, 85, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
