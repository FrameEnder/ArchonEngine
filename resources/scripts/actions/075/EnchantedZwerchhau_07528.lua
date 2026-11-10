-- Enchanted Zwerchhau (Lv. 35): 190, 380 after Enchanted Riposte; 15 White and 15 Black Mana; a Mana Stack.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmEnchanted(effects, player, 15, in_combo and 380 or tierPotency(player, { { 94, 190 }, { 84, 150 }, { 1, 100 } }))

    return effects
end
