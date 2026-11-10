-- Heart Of Corundum (Lv. 82): 15 % less damage taken, 8 s, Clarity of Corundum (4 s) and Catharsis of Corundum (900 at half HP, 20 s; here at its end).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(GNB_EFFECT_HEART_OF_CORUNDUM, 0, 8.0)
    effects:gain_effect(GNB_EFFECT_CLARITY_OF_CORUNDUM, 0, 4.0)
    effects:gain_effect(GNB_EFFECT_CATHARSIS_OF_CORUNDUM, 0, 20.0)

    return effects
end
