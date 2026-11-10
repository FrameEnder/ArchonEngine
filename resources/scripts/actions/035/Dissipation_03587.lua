-- Dissipation (Lv. 60): three Aetherflow and 20 % more healing, 30 s (the faerie leaves: no pets yet).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_aetherflow(3)
    effects:gain_effect_self(SCH_EFFECT_DISSIPATION, 0, 30.0)

    return effects
end
