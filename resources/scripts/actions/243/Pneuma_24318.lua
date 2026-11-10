-- Pneuma (Lv. 90): 380 to the first enemy in a line, 40 % less to the rest; heals the Sage's party 600 (here the Sage).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeMagic(effects, player, 380, 40)
    effects:heal_self(healAmount(player, levelPotency(player, 600, LEVEL_MAIM_AND_MEND)))

    return effects
end
