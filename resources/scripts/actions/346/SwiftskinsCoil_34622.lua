-- Swiftskins Coil (Lv. 65): 630; Swiftscaled and Swiftskin's Venom (30 s); 5 Serpent Offerings from 90.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 630)
    effects:gain_effect_self(VPR_EFFECT_SWIFTSCALED, 0, 40.0)
    effects:gain_effect_self(VPR_EFFECT_SWIFTSKINS_VENOM, 0, 30.0)
    vprOfferings(effects, player, 5)

    return effects
end
