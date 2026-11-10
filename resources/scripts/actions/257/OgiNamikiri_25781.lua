-- Ogi Namikiri (Lv. 90): a sure critical 1,000 to the first enemy in a cone, 40 % less to the rest; a Meditation stack; Ogi Namikiri Ready spent, Kaeshi: Namikiri next.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(40)
    effects:guarantee_critical()
    samDamage(effects, player, 1000)
    effects:gauge_add("meditation", 1)
    effects:consume_effect(SAM_EFFECT_OGI_NAMIKIRI_READY)
    effects:gauge_set("kaeshi", 4)

    return effects
end
