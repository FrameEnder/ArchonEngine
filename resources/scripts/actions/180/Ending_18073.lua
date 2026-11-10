-- Ending (Lv. 60): ends the partnership.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(DNC_EFFECT_CLOSED_POSITION) ~= nil then
        effects:consume_effect(DNC_EFFECT_CLOSED_POSITION)
    end

    return effects
end
