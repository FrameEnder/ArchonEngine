-- Fated Brand (Lv. 96): 120 to all enemies near; Ready to Raze spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbContinuation(effects, player, GNB_EFFECT_READY_TO_RAZE, 120)

    return effects
end
