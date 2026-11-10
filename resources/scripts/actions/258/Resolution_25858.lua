-- Resolution (Lv. 90): 850 to the first enemy in a line, 55 % less to the rest, after Scorch; 4 White and 4 Black Mana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(55)
    rdmMagic(effects, player, 850)
    rdmMana(effects, 4, 4)

    return effects
end
