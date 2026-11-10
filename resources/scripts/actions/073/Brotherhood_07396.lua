-- Brotherhood (Lv. 70): 5 % more damage for the party near (20 s); Meditative Brotherhood (20 s): chakra open more often.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(MNK_EFFECT_BROTHERHOOD, 0, 20.0)
    effects:gain_effect_self(MNK_EFFECT_MEDITATIVE_BROTHERHOOD, 0, 20.0)

    return effects
end
