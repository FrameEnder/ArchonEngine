-- Curing Waltz (Lv. 52): heals the Dancer and the party near 300.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, 300))

    return effects
end
