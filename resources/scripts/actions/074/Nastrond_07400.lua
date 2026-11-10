-- Nastrond (Lv. 70): 720 to the first enemy in a line, 50 % less to the rest; only with Nastrond Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, 720))
    effects:consume_effect(DRG_EFFECT_NASTROND_READY)

    return effects
end
