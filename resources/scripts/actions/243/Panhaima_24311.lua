-- Panhaima (Lv. 80): a barrier and five Panhaimatinon for the party near, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SGE_EFFECT_PANHAIMA, 0, 15.0)

    return effects
end
