-- Radiant Encore (Lv. 100): 700, 800 or 1,100 for the Codas Radiant Finale spent, 50 % less round the target; only with Radiant Encore Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local ready = player:get_effect(BRD_EFFECT_RADIANT_ENCORE_READY)
    local codas = ready and ready.param or 1
    local potency = ({ 700, 800, 1100 })[math.max(1, math.min(codas, 3))]
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, potency, TRAITS_ARC_ACTION_DAMAGE)))
    effects:consume_effect(BRD_EFFECT_RADIANT_ENCORE_READY)

    return effects
end
