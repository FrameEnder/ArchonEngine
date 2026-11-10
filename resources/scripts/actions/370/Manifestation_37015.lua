-- Manifestation (Lv. 100): heals 360 with Galvanize (and Catalyze on a critical), 30 s; only under Seraphism.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, classPotency(player, 360, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gain_effect(SCH_EFFECT_GALVANIZE, 0, 30.0)

    return effects
end
