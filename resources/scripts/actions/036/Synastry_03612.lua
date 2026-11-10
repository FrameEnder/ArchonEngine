-- Synastry (Lv. 50): 40 % of single-target heals to the partner too, 20 s (the echo isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(AST_EFFECT_SYNASTRY, 0, 20.0)

    return effects
end
