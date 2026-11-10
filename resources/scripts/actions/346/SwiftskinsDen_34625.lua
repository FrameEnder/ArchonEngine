-- Swiftskins Den (Lv. 70): 300 to all enemies near; Swiftscaled and Fellskin's Venom; 5 Serpent Offerings from 90.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 300)
    effects:gain_effect_self(VPR_EFFECT_SWIFTSCALED, 0, 40.0)
    effects:gain_effect_self(VPR_EFFECT_FELLSKINS_VENOM, 0, 30.0)
    vprOfferings(effects, player, 5)

    return effects
end
