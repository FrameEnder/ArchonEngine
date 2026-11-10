-- Oka (Lv. 45): 100 to all enemies near, 120 after Fuga / Fuko with Fuka, Ka and 10 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samStep(effects, player, in_combo, 100, 120, 4, 10, SAM_EFFECT_FUKA)

    return effects
end
