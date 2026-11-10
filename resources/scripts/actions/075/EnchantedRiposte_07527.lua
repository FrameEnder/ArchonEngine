-- Enchanted Riposte (Lv. 1): 340; 20 White and 20 Black Mana (or Magicked Swordplay); a Mana Stack from 68.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmEnchanted(effects, player, 20, tierPotency(player, { { 94, 340 }, { 84, 280 }, { 1, 220 } }))

    return effects
end
