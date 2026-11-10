-- Primal Rend (Lv. 90): a leap to the target: 720 to it, 50 % less round it, a critical direct hit; only with Primal Rend Ready; gives Primal Ruination Ready (20 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    effects:guarantee_direct_hit()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 720))
    effects:consume_effect(WAR_EFFECT_PRIMAL_REND_READY)
    if player.level >= 100 then
        effects:gain_effect_self(WAR_EFFECT_PRIMAL_RUINATION_READY, 0, 20.0)
    end

    return effects
end
