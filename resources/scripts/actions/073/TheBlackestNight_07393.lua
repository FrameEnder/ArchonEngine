-- The Blackest Night (Lv. 70): a barrier of 25 % of the target's HP, 7 s (Dark Arts comes as it's cast, not as the barrier is used up).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(DRK_EFFECT_BLACKEST_NIGHT, 0, 7.0)
    effects:barrier_percent(DRK_EFFECT_BLACKEST_NIGHT, 25)
    effects:gauge_set("dark_arts", 1)

    return effects
end
