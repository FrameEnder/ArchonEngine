-- Scorch (Lv. 80): 750 to the first enemy, 55 % less round it, after Verflare or Verholy; 4 White and 4 Black Mana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(55)
    rdmMagic(effects, player, 750)
    rdmMana(effects, 4, 4)

    return effects
end
