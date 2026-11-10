-- Hypervelocity (Lv. 86): 180; Ready to Blast spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbContinuation(effects, player, GNB_EFFECT_READY_TO_BLAST, 180)

    return effects
end
