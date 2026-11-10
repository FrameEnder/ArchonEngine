-- Hissatsu Yaten (Lv. 56): 100 and a step back; Enhanced Enpi, 15 s; 10 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 100)
    effects:gauge_add("kenki", -10)
    effects:gain_effect_self(SAM_EFFECT_ENHANCED_ENPI, 0, 15.0)

    return effects
end
