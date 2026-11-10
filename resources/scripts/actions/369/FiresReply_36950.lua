-- Fires Reply (Lv. 100): 1,400 to the target, 35 % less round it; only with Fire's Rumination; Formless Fist.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(35)
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 1400))
    effects:consume_effect(MNK_EFFECT_FIRES_RUMINATION)
    effects:gain_effect_self(MNK_EFFECT_FORMLESS_FIST, 0, 30.0)

    return effects
end
