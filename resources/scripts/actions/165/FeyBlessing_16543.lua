-- Fey Blessing (Lv. 76): the faerie heals the party near 320 (the Scholar does: no pets yet).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, classPotency(player, 320, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
