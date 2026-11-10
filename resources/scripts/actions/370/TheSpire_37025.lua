-- The Spire (Lv. 30): a barrier of a 400 heal, 30 s (barriers don't absorb yet).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(AST_EFFECT_THE_SPIRE, 0, 30.0)
    astPlay(effects, player, 6)

    return effects
end
