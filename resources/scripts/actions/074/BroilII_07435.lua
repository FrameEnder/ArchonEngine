-- Broil I I (Lv. 64): 240.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 240, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
