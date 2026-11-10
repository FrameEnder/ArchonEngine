-- Psyche (Lv. 92): 690 to the first enemy, 50 % less round it.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeMagic(effects, player, 690, 50)

    return effects
end
