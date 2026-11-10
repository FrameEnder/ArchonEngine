-- Harpe (Lv. 15): 300; Enhanced Harpe spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 300, true)
    if player:get_effect(RPR_EFFECT_ENHANCED_HARPE) ~= nil then
        effects:consume_effect(RPR_EFFECT_ENHANCED_HARPE)
    end

    return effects
end
