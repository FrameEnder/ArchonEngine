-- Improvised Finish (Lv. 80): a barrier for the party near (barriers don't absorb yet); Improvisation over.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(DNC_EFFECT_IMPROVISATION) ~= nil then
        effects:consume_effect(DNC_EFFECT_IMPROVISATION)
    end

    return effects
end
