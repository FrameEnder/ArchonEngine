-- Resurrection (Lv. 12).
-- NOTE: reviving players isn't implemented yet (as Raise).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    player:send_message("Resurrection isn't implemented yet.")

    return effects
end
