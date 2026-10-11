-- Divine Benison (Lv. 66)
-- NOTE: The barrier icon is applied, but barriers don't absorb damage yet.
DURATION = 15.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_DIVINE_BENISON, 0, DURATION)

    return effects
end
