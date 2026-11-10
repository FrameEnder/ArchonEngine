-- Jolt I I (Lv. 62): 280; 2 White and 2 Black Mana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmMagic(effects, player, 280)
    rdmMana(effects, 2, 2)
    rdmDualcast(effects, player)

    return effects
end
