-- Winds Reply (Lv. 96): 1,040 to the first enemy in a line, 35 % less to the rest; only with Wind's Rumination.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(35)
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 1040))
    effects:consume_effect(MNK_EFFECT_WINDS_RUMINATION)

    return effects
end
