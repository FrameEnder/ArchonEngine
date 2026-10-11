-- Surecast (Lv. 44) - role action
-- NOTE: The status is applied, but there are no interrupts or knockbacks to prevent yet.
DURATION = 6.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_SURECAST, 0, DURATION)

    return effects
end
