-- Tenri Jindo (Lv. 100): 1,100 to the target and round it; only with Tenri Jindo Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 1100))
    effects:consume_effect(NIN_EFFECT_TENRI_JINDO_READY)

    return effects
end
