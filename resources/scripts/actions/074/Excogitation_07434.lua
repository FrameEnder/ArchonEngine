-- Excogitation (Lv. 62): heals 800 when the target falls under half HP or after 45 s (here: at once); an Aetherflow (none under Recitation).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(SCH_EFFECT_RECITATION) ~= nil then
        effects:guarantee_critical()
    end
    effects:heal(healAmount(player, classPotency(player, 800, TRAITS_ACN_MAIM_AND_MEND)))
    schSpendAetherflow(effects, player, true)

    return effects
end
