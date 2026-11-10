-- Aetherpact (Lv. 70): Fey Union on a party member (300 over time while the Faerie Gauge lasts; here 10 of it and the status, 30 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_add("fairy", -10)
    effects:gain_effect(SCH_EFFECT_FEY_UNION, 0, 30.0)

    return effects
end
