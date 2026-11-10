-- Sunflare (Lv. 100): 1,000 to the target, 50 % less round it; only in Lightwyrm Trance.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 1000, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
