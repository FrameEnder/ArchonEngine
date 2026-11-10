-- Horoscope (Lv. 76): Horoscope for the party near, 10 s (its heal comes with Horoscope's end, 16558).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(AST_EFFECT_HOROSCOPE, 0, 10.0)

    return effects
end
