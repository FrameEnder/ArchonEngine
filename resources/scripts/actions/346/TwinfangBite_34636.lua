-- Twinfang Bite (Lv. 75): 120, 170 with Hunter's Venom.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 120 + vprBonus(effects, player, VPR_EFFECT_HUNTERS_VENOM, 50))

    return effects
end
