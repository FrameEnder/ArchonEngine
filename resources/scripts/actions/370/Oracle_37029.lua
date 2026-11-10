-- Oracle (Lv. 92): 860 to the target, 50 % less round it; Divining spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, 860, 50)
    effects:consume_effect(AST_EFFECT_DIVINING)

    return effects
end
