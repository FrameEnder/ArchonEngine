-- Passage Of Arms (Lv. 70): blocks everything for 18 s; those behind take 15 % less.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(1175, 0, 18.0)

    return effects
end
