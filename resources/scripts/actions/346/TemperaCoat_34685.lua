-- Tempera Coat (Lv. 10): a barrier of 20 % of the Pictomancer's HP, 10 s (barriers don't absorb yet).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(PCT_EFFECT_TEMPERA_COAT, 0, 10.0)

    return effects
end
