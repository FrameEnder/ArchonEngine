-- Broil I V (Lv. 82): 320.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 320, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
