-- Vercure (Lv. 54): heals 350.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, levelPotency(player, 350, LEVEL_MAIM_AND_MEND)))
    rdmDualcast(effects, player)

    return effects
end
