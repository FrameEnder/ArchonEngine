-- Last Dance (Lv. 92): 540 to the target, 60 % less round it; Last Dance Ready spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    dncDamage(effects, player, 540)
    effects:consume_effect(DNC_EFFECT_LAST_DANCE_READY)

    return effects
end
