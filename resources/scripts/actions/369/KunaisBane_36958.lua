-- Kunais Bane (Lv. 92): 700 to the target and round it; it takes 10 % more from the Ninja for 15 s; only while Hidden (or under Shadow Walker), which it ends.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 700))
    effects:gain_effect(NIN_EFFECT_KUNAIS_BANE, 0, 15.0)
    ninLeaveHiding(effects, player)

    return effects
end
