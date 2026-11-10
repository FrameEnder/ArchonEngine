-- Regress (Lv. 74): back to the Hellsgate (movement isn't done); Threshold spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:consume_effect(RPR_EFFECT_THRESHOLD)

    return effects
end
