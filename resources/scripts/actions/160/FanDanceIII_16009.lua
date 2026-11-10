-- Fan Dance I I I (Lv. 66): 220 to the target, 60 % less round it; Threefold Fan Dance spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    dncDamage(effects, player, 220)
    effects:consume_effect(DNC_EFFECT_THREEFOLD_FAN_DANCE)

    return effects
end
