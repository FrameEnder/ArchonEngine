-- Tempera Grassa (Lv. 88): Tempera Coat for a barrier on the party near, 10 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:consume_effect(PCT_EFFECT_TEMPERA_COAT)
    effects:gain_effect(PCT_EFFECT_TEMPERA_GRASSA, 0, 10.0)

    return effects
end
