-- Pepsis (Lv. 58): the Eukrasian barriers turned to heals: 450 (Diagnosis) or 350 (Prognosis); here 350 to the Sage.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_self(healAmount(player, levelPotency(player, 350, LEVEL_MAIM_AND_MEND)))

    return effects
end
