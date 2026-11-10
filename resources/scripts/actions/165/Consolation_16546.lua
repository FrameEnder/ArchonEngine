-- Consolation (Lv. 80): Seraph heals the party near 250 and gives as much again as a barrier (Galvanize stands in), 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, classPotency(player, 250, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gain_effect(SCH_EFFECT_GALVANIZE, 0, 30.0)

    return effects
end
