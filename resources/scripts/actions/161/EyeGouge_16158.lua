-- Eye Gouge (Lv. 70): 300; Ready to Gouge spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbContinuation(effects, player, GNB_EFFECT_READY_TO_GOUGE, 300)

    return effects
end
