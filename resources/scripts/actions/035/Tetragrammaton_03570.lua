-- Tetragrammaton (Lv. 60)
POTENCY = 700

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:heal(whmHeal(player, POTENCY))

    return effects
end
