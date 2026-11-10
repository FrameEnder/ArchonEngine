-- Kaeshi Goken (Lv. 74): 300 to all enemies near; Tsubame-gaeshi spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 300)
    effects:consume_effect(SAM_EFFECT_TSUBAME_READY)
    effects:gauge_set("kaeshi", 0)

    return effects
end
