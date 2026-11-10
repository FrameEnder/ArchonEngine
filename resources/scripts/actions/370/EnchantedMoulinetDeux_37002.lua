-- Enchanted Moulinet Deux (Lv. 52): 140 to the enemies in a cone after Enchanted Moulinet; 15 and 15 Mana; a Mana Stack.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmEnchanted(effects, player, 15, 140)

    return effects
end
