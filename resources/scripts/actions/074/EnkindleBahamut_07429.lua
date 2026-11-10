-- Enkindle Bahamut (Lv. 70): Demi-Bahamut's Akh Morn: 1,300 to the target, 50 % less round it.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 1300, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
