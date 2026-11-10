-- Accession (Lv. 100): heals the Scholar and the party near 240 with Galvanize, 30 s; only under Seraphism.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, classPotency(player, 240, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gain_effect(SCH_EFFECT_GALVANIZE, 0, 30.0)

    return effects
end
