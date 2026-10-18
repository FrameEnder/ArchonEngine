-- Provoke (Lv. 15) - role action: puts you at the top of the target's enmity.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:provoke()

    return effects
end
