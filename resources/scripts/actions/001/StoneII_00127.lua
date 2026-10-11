-- Stone II (Lv. 18)
POTENCY = 190

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, POTENCY))

    return effects
end
