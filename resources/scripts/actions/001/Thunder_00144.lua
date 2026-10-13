-- Thunder (Lv. 6)
POTENCY = 100
DOT_POTENCY = 45
DURATION = 24.0
EFFECT_THUNDER = 161

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:damage_over_time(EFFECT_THUNDER, DURATION, player.parameters:calc_magical_damage(DOT_POTENCY))

    return effects
end
