-- Natures Minne (Lv. 66): the party near gets 15 % more from heals, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(1202, 0, 15.0)

    return effects
end
