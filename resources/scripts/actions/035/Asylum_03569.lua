-- Asylum (Lv. 52)
-- NOTE: Asylum is ground-targeted, and Kawari doesn't handle ground-targeted action requests yet, so this won't run until it does.
-- When it does, this gives the target the Asylum healing-over-time effect. The 10% healing bonus isn't implemented.
POTENCY = 100
DURATION = 24.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:heal_over_time(EFFECT_ASYLUM, DURATION, whmHeal(player, POTENCY))

    return effects
end
