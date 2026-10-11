-- Swiftcast (Lv. 18) - role action
-- The next spell with a cast time is cast instantly.
DURATION = 10.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_SWIFTCAST, 0, DURATION)

    return effects
end
