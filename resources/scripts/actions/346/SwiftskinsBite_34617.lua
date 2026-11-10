-- Swiftskins Bite (Lv. 45): 180 to all enemies near; Swiftscaled.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 180)
    effects:gain_effect_self(VPR_EFFECT_SWIFTSCALED, 0, 40.0)

    return effects
end
