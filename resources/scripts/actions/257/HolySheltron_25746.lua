-- Holy Sheltron (Lv. 82): 15 % less damage for 8 s, Knight's Resolve (4 s) and Knight's Benediction (12 s); 50 Oath.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(2674, 0, 8.0)
    effects:gain_effect_self(2675, 0, 4.0)
    effects:gain_effect_self(2676, 0, 12.0)
    effects:gauge_add("oath", -50)

    return effects
end
