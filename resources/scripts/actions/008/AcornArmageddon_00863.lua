-- Frenzied Elm (Chasing Shadows.) Makes up to two targets Heavy (status 14, param 50) for about 14 seconds, with no
-- damage, as recorded on retail.
EFFECT_HEAVY = 14
DURATION = 14.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_HEAVY, 50, DURATION)

    return effects
end
