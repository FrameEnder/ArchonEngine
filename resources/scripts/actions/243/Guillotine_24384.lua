-- Guillotine (Lv. 70): 200 to the enemies in a cone; 10 Shroud from 80.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprReaver(effects, player, 200, 200, nil, nil)

    return effects
end
