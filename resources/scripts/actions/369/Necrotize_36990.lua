-- Necrotize (Lv. 92): 500; an Aetherflow.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 500, TRAITS_ACN_MAIM_AND_MEND)))
    effects:spend_aetherflow()

    return effects
end
