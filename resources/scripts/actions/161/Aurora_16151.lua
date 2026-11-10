-- Aurora (Lv. 45): Regen of 300, 18 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_over_time(GNB_EFFECT_AURORA, 18.0, healAmount(player, 300))

    return effects
end
