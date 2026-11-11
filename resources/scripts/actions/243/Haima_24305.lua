-- Haima (Lv. 70): a barrier and five Haimatinon, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SGE_EFFECT_HAIMA, 0, 15.0)
    -- A barrier of a 300-potency heal (the job guide); the five Haimatinon renewing it aren't played out yet.
    effects:barrier(SGE_EFFECT_HAIMA, healAmount(player, 300))

    return effects
end
