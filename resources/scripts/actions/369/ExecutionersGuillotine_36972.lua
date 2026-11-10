-- Executioners Guillotine (Lv. 96): 260 to the enemies in a cone; 10 Shroud.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprReaver(effects, player, 260, 260, nil, nil)

    return effects
end
