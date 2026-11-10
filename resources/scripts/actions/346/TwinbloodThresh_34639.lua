-- Twinblood Thresh (Lv. 80): 50 to all enemies near, 80 with Fellskin's Venom.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 50 + vprBonus(effects, player, VPR_EFFECT_FELLSKINS_VENOM, 30))

    return effects
end
