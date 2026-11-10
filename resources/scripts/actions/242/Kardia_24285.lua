-- Kardia (Lv. 4): Kardia, and Kardion on the target (its heals after the Sage's attacks aren't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SGE_EFFECT_KARDIA, 0, 0.0)
    effects:gain_effect(SGE_EFFECT_KARDION, 0, 0.0)

    return effects
end
