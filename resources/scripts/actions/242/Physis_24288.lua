-- Physis (Lv. 20): 100 over 15 s for the party near.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_over_time(SGE_EFFECT_PHYSIS, 15.0, healAmount(player, levelPotency(player, 100, LEVEL_MAIM_AND_MEND)))

    return effects
end
