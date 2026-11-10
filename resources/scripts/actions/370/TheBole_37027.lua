-- The Bole (Lv. 30): 10 % less damage taken, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(AST_EFFECT_THE_BOLE, 0, 15.0)
    astPlay(effects, player, 2)

    return effects
end
