-- Frenzied Elm (Chasing Shadows.) Makes up to two targets Heavy (status 14, param 50) for 15 seconds, with no
-- damage, as recorded on retail (Bob's Chronofoil captures: 15.0 s, 4 samples.)
EFFECT_HEAVY = 14
DURATION = 15.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_HEAVY, 50, DURATION)

    return effects
end
