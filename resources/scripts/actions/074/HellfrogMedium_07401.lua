-- Hellfrog Medium (Lv. 62): 250 to the target and round it; 50 Ninki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 250))
    effects:gauge_add("ninki", -50)

    return effects
end
