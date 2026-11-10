-- Perfectio (Lv. 100): 1,300 to the target, 20 % less round it; Perfectio Parata spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(20)
    rprDamage(effects, player, 1300, true)
    effects:consume_effect(RPR_EFFECT_PERFECTIO_PARATA)

    return effects
end
