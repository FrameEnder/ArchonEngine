-- Seraphism (Lv. 100): Seraphism, 20 s: Adloquium becomes Manifestation, Concitation Accession; the party near heals 100 over time.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SCH_EFFECT_SERAPHISM, 0, 20.0)
    effects:heal_over_time(SCH_EFFECT_SERAPHISM_REGEN, 20.0, healAmount(player, classPotency(player, 100, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
