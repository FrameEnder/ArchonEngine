-- Indomitability (Lv. 52): heals the Scholar and the party near 400; an Aetherflow (none under Recitation, whose heal is then critical).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(SCH_EFFECT_RECITATION) ~= nil then
        effects:guarantee_critical()
    end
    effects:heal(healAmount(player, classPotency(player, 400, TRAITS_ACN_MAIM_AND_MEND)))
    schSpendAetherflow(effects, player, true)

    return effects
end
