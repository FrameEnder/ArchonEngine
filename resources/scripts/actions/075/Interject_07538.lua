-- Interject (Lv. 18) - role action: interrupts the target's cast.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:interrupt()

    return effects
end
