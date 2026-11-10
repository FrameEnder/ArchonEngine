-- Tendo Kaeshi Setsugekka (Lv. 100): a sure critical 1,100; Tsubame-gaeshi spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    samDamage(effects, player, 1100)
    effects:consume_effect(SAM_EFFECT_TSUBAME_READY)
    effects:gauge_set("kaeshi", 0)

    return effects
end
