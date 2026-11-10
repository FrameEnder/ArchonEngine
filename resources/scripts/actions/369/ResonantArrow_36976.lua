-- Resonant Arrow (Lv. 96): 640 to the target, 50 % less round it; only with Resonant Arrow Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 640, TRAITS_ARC_ACTION_DAMAGE)))
    effects:consume_effect(BRD_EFFECT_RESONANT_ARROW_READY)

    return effects
end
