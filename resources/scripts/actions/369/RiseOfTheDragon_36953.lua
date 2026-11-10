-- Rise Of The Dragon (Lv. 92): 550 to the first enemy round it, 50 % less to the rest; only with Dragon's Flight.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, 550))
    effects:consume_effect(DRG_EFFECT_DRAGONS_FLIGHT)

    return effects
end
