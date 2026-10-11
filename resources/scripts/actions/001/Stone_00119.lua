-- Stone (Lv. 1)
POTENCY = 140

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, POTENCY))

    return effects
end
