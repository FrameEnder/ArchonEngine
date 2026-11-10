-- Enlightenment (Lv. 74): 160 to the enemies in a line; spends the five chakra.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 160))
    effects:gauge_add("chakra", -5)

    return effects
end
