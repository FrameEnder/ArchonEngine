-- Divine Caress (Lv. 100)
-- Grants a barrier to the caster and party members within 30 yalms. Uses up Divine Grace.
-- NOTE: Divine Aura isn't granted when it expires.
DURATION = 10.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_DIVINE_CARESS, 0, DURATION)
    -- A barrier of a 400-potency heal (the 7.x tooltip.)
    effects:barrier(EFFECT_DIVINE_CARESS, healAmount(player, 400))
    effects:consume_effect(EFFECT_DIVINE_GRACE)

    return effects
end
