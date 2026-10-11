-- Aero II (Lv. 46)
POTENCY = 50
DOT_POTENCY = 50
DURATION = 30.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, POTENCY))
    effects:damage_over_time(EFFECT_AERO_II, DURATION, whmDamage(player, DOT_POTENCY))

    return effects
end
