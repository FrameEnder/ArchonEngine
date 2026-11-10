-- Embolden (Lv. 58): the party near deals 5 % more damage, 20 s; Thorned Flourish (30 s) from 92.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(RDM_EFFECT_EMBOLDEN, 0, 20.0)
    if player.level >= 92 then
        effects:gain_effect_self(RDM_EFFECT_THORNED_FLOURISH, 0, 30.0)
    end

    return effects
end
