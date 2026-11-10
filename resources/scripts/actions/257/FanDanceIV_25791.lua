-- Fan Dance I V (Lv. 86): 460 to the first enemy in a cone, 60 % less to the rest; Fourfold Fan Dance spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    dncDamage(effects, player, 460)
    effects:consume_effect(DNC_EFFECT_FOURFOLD_FAN_DANCE)

    return effects
end
