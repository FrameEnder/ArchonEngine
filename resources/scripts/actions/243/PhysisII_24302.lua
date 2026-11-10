-- Physis I I (Lv. 60): 130 over 15 s and 10 % more healing received for the party near.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_over_time(SGE_EFFECT_PHYSIS_II, 15.0, healAmount(player, levelPotency(player, 130, LEVEL_MAIM_AND_MEND)))

    return effects
end
