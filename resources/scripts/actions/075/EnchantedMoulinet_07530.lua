-- Enchanted Moulinet (Lv. 52): 130 to the enemies in a cone; 20 White and 20 Black Mana; a Mana Stack.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmEnchanted(effects, player, 20, 130)

    return effects
end
