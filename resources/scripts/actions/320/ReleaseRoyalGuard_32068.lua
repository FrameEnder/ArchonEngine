-- Release Royal Guard (Lv. 10): ends Royal Guard.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(GNB_EFFECT_ROYAL_GUARD) ~= nil then
        effects:consume_effect(GNB_EFFECT_ROYAL_GUARD)
    end

    return effects
end
