-- Searing Flash (Lv. 96): 700 to the target and round it; only with Ruby's Glimmer.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 700, TRAITS_ACN_MAIM_AND_MEND)))
    effects:consume_effect(SMN_EFFECT_RUBYS_GLIMMER)

    return effects
end
