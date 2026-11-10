-- Earths Reply (Lv. 64): heals the Monk and the party near 300 (and 500 more from Earth's Resolve, here at once); only with Earth's Rumination.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, 800))
    effects:consume_effect(MNK_EFFECT_EARTHS_RUMINATION)

    return effects
end
