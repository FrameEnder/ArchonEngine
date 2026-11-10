-- Steeled Meditation (Lv. 15): opens a chakra.
-- One more of the five chakra (the job gauge's "chakra"; critical weaponskills open them too from 38, server/action.rs).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_add("chakra", 1)

    return effects
end
