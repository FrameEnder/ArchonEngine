-- Pirouette (Lv. 15): a step (4); done if it's the one due.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncStep(effects, player, 4)

    return effects
end
