-- Meisui (Lv. 72): Shadow Walker into 50 Ninki and Meisui (30 s); only under Shadow Walker.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:consume_effect(NIN_EFFECT_SHADOW_WALKER)
    ninGainNinki(effects, player, 50)
    effects:gain_effect_self(NIN_EFFECT_MEISUI, 0, 30.0)

    return effects
end
