-- Enpi (Lv. 15): 100, 270 with Enhanced Enpi; 10 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(SAM_EFFECT_ENHANCED_ENPI) ~= nil then
        samDamage(effects, player, 270)
        effects:consume_effect(SAM_EFFECT_ENHANCED_ENPI)
    else
        samDamage(effects, player, 100)
    end
    samKenki(effects, player, 10)

    return effects
end
