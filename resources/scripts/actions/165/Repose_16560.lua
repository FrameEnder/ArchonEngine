-- Repose (Lv. 8) - role action
-- NOTE: The sleep icon is applied, but enemy AI doesn't respect sleep yet.
DURATION = 30.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_SLEEP, 0, DURATION)

    return effects
end
