-- The Ewer (Lv. 30): 200 over 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_over_time(AST_EFFECT_THE_EWER, 15.0, healAmount(player, levelPotency(player, 200, LEVEL_MAIM_AND_MEND)))
    astPlay(effects, player, 5)

    return effects
end
