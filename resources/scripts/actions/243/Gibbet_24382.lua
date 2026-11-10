-- Gibbet (Lv. 70): 500 (560 with Enhanced Gibbet); Enhanced Gallows (60 s); 10 Shroud from 80.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprReaver(effects, player, 500, 560, RPR_EFFECT_ENHANCED_GIBBET, RPR_EFFECT_ENHANCED_GALLOWS)

    return effects
end
