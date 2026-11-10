-- Kardia End (Lv. 4): Kardia ended.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(SGE_EFFECT_KARDIA) ~= nil then
        effects:consume_effect(SGE_EFFECT_KARDIA)
    end

    return effects
end
