-- Enchanted Redoublement Swordplay (Lv. 60): Enchanted Redoublement as retail's client sent it after Manafication (Bob's captures, Red Mages at 80-89: 45960-45962 four times each, as many as Manafication's Magicked Swordplay stacks allow); as 7529.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmEnchanted(effects, player, 15, in_combo and 560 or 190)

    return effects
end
