-- Jagged Maw (Lv. 50): 180 to all enemies near, 40 more with Grimhunter's Venom; Grimskin's Venom; 10 Serpent Offerings from 90.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 180 + vprBonus(effects, player, VPR_EFFECT_GRIMHUNTERS_VENOM, 40))
    vprVenom(effects, player, VPR_EFFECT_GRIMSKINS_VENOM)
    vprOfferings(effects, player, 10)

    return effects
end
