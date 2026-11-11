-- Divine Benison (Lv. 66)
DURATION = 15.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_DIVINE_BENISON, 0, DURATION)
    -- A barrier of a 500-potency heal (the 7.x tooltip.)
    effects:barrier(EFFECT_DIVINE_BENISON, healAmount(player, 500))

    return effects
end
