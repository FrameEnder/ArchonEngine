-- Twinfang Thresh (Lv. 80): 50 to all enemies near, 80 with Fellhunter's Venom.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 50 + vprBonus(effects, player, VPR_EFFECT_FELLHUNTERS_VENOM, 30))

    return effects
end
