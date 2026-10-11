-- Stone IV (Lv. 64)
POTENCY = 260

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, POTENCY))

    return effects
end
