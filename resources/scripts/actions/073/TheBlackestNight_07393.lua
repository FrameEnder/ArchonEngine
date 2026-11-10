-- The Blackest Night (Lv. 70): a barrier of 25 % of the target's HP, 7 s (barriers don't absorb yet, so Dark Arts comes as it's cast).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(DRK_EFFECT_BLACKEST_NIGHT, 0, 7.0)
    effects:gauge_set("dark_arts", 1)

    return effects
end
