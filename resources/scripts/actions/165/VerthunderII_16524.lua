-- Verthunder I I (Lv. 18): 140 to the target and round it; 7 Black Mana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmMagic(effects, player, 140)
    rdmMana(effects, 0, 7)
    rdmDualcast(effects, player)

    return effects
end
