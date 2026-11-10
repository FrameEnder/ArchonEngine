-- Kaeshi Namikiri (Lv. 90): a sure critical 1,000 to the first enemy in a cone, 40 % less to the rest.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(40)
    effects:guarantee_critical()
    samDamage(effects, player, 1000)
    effects:gauge_set("kaeshi", 0)

    return effects
end
