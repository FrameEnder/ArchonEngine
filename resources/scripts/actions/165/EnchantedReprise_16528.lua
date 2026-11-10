-- Enchanted Reprise (Lv. 76): 420; 5 White and 5 Black Mana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_add("white_mana", -5)
    effects:gauge_add("black_mana", -5)
    effects:damage(DAMAGE_TYPE_MAGIC, physicalDamage(player, 420))

    return effects
end
