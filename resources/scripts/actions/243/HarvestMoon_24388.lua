-- Harvest Moon (Lv. 82): 800 to the target, 40 % less round it; 10 Soul; Soulsow spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(40)
    rprDamage(effects, player, 800, true)
    rprSoul(effects, player, 10)
    effects:consume_effect(RPR_EFFECT_SOULSOW)

    return effects
end
