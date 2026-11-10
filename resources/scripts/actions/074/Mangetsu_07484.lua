-- Mangetsu (Lv. 35): 100 to all enemies near, 120 after Fuga / Fuko with Fugetsu, Getsu and 10 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samStep(effects, player, in_combo, 100, 120, 2, 10, SAM_EFFECT_FUGETSU)

    return effects
end
