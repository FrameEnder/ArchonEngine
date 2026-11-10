-- Tendo Setsugekka (Lv. 100): a sure critical 1,100; three Sen and Tendo; a Meditation stack; Tsubame-gaeshi.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    samDamage(effects, player, 1100)
    effects:consume_effect(SAM_EFFECT_TENDO)
    samIaijutsu(effects, player, 3)

    return effects
end
