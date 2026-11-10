-- Sacrificium (Lv. 92): 700 to the target, 20 % less round it; Oblatio spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(20)
    rprDamage(effects, player, 700, true)
    effects:consume_effect(RPR_EFFECT_OBLATIO)

    return effects
end
