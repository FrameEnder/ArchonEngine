-- Abdomen Tear (Lv. 70): 260; Ready to Tear spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbContinuation(effects, player, GNB_EFFECT_READY_TO_TEAR, 260)

    return effects
end
