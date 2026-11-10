-- Starcross (Lv. 100): 1,000 to the first enemy round it, 40 % less to the rest; only with Starcross Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(40)
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, 1000))
    effects:consume_effect(DRG_EFFECT_STARCROSS_READY)

    return effects
end
