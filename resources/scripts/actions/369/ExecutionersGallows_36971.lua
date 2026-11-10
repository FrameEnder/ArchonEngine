-- Executioners Gallows (Lv. 96): 700 (760 with Enhanced Gallows); Enhanced Gibbet; 10 Shroud.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprReaver(effects, player, 700, 760, RPR_EFFECT_ENHANCED_GALLOWS, RPR_EFFECT_ENHANCED_GIBBET)

    return effects
end
