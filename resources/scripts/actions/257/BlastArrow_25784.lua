-- Blast Arrow (Lv. 86): 700 to the first enemy in a line, 50 % less to the rest; only with Blast Arrow Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 700, TRAITS_ARC_ACTION_DAMAGE)))
    effects:consume_effect(BRD_EFFECT_BLAST_ARROW_READY)

    return effects
end
