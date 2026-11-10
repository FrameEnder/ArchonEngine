-- Concitation (Lv. 96): heals the Scholar and the party near 200, with Galvanize, 30 s (free and critical under Recitation).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(SCH_EFFECT_RECITATION) ~= nil then
        effects:guarantee_critical()
        effects:consume_effect(SCH_EFFECT_RECITATION)
    end
    effects:heal(healAmount(player, classPotency(player, 200, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gain_effect(SCH_EFFECT_GALVANIZE, 0, 30.0)

    return effects
end
