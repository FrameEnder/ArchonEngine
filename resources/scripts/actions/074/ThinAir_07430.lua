-- Thin Air (Lv. 58)
-- The next action that costs MP is free.
DURATION = 12.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_THIN_AIR, 0, DURATION)

    return effects
end
