-- Uncoiled Twinblood (Lv. 92): 120 to the target, 75 % less round it, 170 when Poised.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(75)
    vprDamage(effects, player, 120 + vprBonus(effects, player, VPR_EFFECT_POISED_FOR_TWINBLOOD, 50))

    return effects
end
