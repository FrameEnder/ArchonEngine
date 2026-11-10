-- The Arrow (Lv. 30): 10 % more healing received, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(AST_EFFECT_THE_ARROW, 0, 15.0)
    astPlay(effects, player, 3)

    return effects
end
