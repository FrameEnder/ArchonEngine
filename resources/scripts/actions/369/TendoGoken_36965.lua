-- Tendo Goken (Lv. 100): 410 to all enemies near; two Sen and Tendo; a Meditation stack; Tsubame-gaeshi.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 410)
    effects:consume_effect(SAM_EFFECT_TENDO)
    samIaijutsu(effects, player, 2)

    return effects
end
