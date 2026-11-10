-- Uncoiled Twinfang (Lv. 92): 120 to the target, 75 % less round it, 170 when Poised; Poised for Twinblood.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(75)
    vprDamage(effects, player, 120 + vprBonus(effects, player, VPR_EFFECT_POISED_FOR_TWINFANG, 50))
    effects:gain_effect_self(VPR_EFFECT_POISED_FOR_TWINBLOOD, 0, 60.0)

    return effects
end
