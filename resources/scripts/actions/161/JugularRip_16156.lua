-- Jugular Rip (Lv. 70): 220; Ready to Rip spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbContinuation(effects, player, GNB_EFFECT_READY_TO_RIP, 220)

    return effects
end
