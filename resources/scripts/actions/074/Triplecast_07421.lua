-- Triplecast (Lv. 66): the next three spells cast at once (15 s; the client's cast times.)
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(1211, 3, 15.0)

    return effects
end
