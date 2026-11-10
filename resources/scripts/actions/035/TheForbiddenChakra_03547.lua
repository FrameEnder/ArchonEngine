-- The Forbidden Chakra (Lv. 54): 400; spends the five chakra.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 400))
    effects:gauge_add("chakra", -5)

    return effects
end
