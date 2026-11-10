-- Veraero I I (Lv. 22): 140 to the target and round it; 7 White Mana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmMagic(effects, player, 140)
    rdmMana(effects, 7, 0)
    rdmDualcast(effects, player)

    return effects
end
