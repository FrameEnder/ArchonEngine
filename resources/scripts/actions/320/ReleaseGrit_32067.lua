-- Release Grit (Lv. 10): ends Grit.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(DRK_EFFECT_GRIT) ~= nil then
        effects:consume_effect(DRK_EFFECT_GRIT)
    end

    return effects
end
