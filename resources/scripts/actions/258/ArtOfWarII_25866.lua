-- Art Of War I I (Lv. 82): 180 to all enemies near.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 180, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
