-- Earthly Star (Lv. 62): the star, 20 s (Earthly Dominance its first 10, Giant Dominance after).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(AST_EFFECT_EARTHLY_DOMINANCE, 0, 20.0)

    return effects
end
