-- Phlegma (Lv. 26): 400 to the first enemy, 30 % less round it.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeMagic(effects, player, 400, 30)

    return effects
end
