-- Primal Ruination (Lv. 100): 800 to the target, 50 % less round it, a critical direct hit; only with Primal Ruination Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    effects:guarantee_direct_hit()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 800))
    effects:consume_effect(WAR_EFFECT_PRIMAL_RUINATION_READY)

    return effects
end
