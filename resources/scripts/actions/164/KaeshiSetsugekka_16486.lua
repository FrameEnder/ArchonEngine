-- Kaeshi Setsugekka (Lv. 74): a sure critical 680; Tsubame-gaeshi spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    samDamage(effects, player, 680)
    effects:consume_effect(SAM_EFFECT_TSUBAME_READY)
    effects:gauge_set("kaeshi", 0)

    return effects
end
