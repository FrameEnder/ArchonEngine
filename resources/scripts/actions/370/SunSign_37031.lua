-- Sun Sign (Lv. 100): the party near takes 10 % less damage, 15 s; Suntouched spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(AST_EFFECT_SUN_SIGN, 0, 15.0)
    effects:consume_effect(AST_EFFECT_SUNTOUCHED)

    return effects
end
