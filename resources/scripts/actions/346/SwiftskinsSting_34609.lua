-- Swiftskins Sting (Lv. 20): 300 after Steel or Reaving Fangs; Swiftscaled (15 % quicker, 40 s; the haste isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 300)
    effects:gain_effect_self(VPR_EFFECT_SWIFTSCALED, 0, 40.0)

    return effects
end
