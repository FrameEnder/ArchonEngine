EFFECT_SHIELD_SAMBA = 1826

function doAction(player, in_combo)
    effects = EffectsBuilder()
    -- The whole party in range gets it on retail (Bob's Chronofoil captures: 20 allies, 3 self.)
    effects:gain_effect(EFFECT_SHIELD_SAMBA, 0, 15.0)

    return effects
end
