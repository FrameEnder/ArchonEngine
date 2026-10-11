-- Divine Caress (Lv. 100)
-- Grants a barrier to the caster and party members within 30 yalms. Uses up Divine Grace.
-- NOTE: The barrier icon is applied, but barriers don't absorb damage yet, and Divine Aura isn't granted when it expires.
DURATION = 10.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_DIVINE_CARESS, 0, DURATION)
    effects:consume_effect(EFFECT_DIVINE_GRACE)

    return effects
end
