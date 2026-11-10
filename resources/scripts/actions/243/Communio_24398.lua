-- Communio (Lv. 90): 1,100 to the target, 20 % less round it; Enshroud over; Perfectio Parata (30 s) from Perfectio Occulta.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(20)
    rprDamage(effects, player, 1100, true)
    effects:consume_effect(RPR_EFFECT_ENSHROUDED)
    effects:gauge_set("lemure", 0)
    effects:gauge_set("void", 0)
    if player:get_effect(RPR_EFFECT_PERFECTIO_OCCULTA) ~= nil then
        effects:consume_effect(RPR_EFFECT_PERFECTIO_OCCULTA)
        effects:gain_effect_self(RPR_EFFECT_PERFECTIO_PARATA, 0, 30.0)
    end

    return effects
end
