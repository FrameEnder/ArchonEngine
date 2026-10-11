-- Benediction (Lv. 50)
function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:heal_to_full()

    return effects
end
