-- Aquaveil (Lv. 86)
-- NOTE: The status is applied, but damage reduction isn't implemented yet.
DURATION = 8.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_AQUAVEIL, 0, DURATION)

    return effects
end
