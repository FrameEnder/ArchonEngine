-- Stone III (Lv. 54)
POTENCY = 220

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, POTENCY))

    return effects
end
