-- Claw Motif (Lv. 96): paints the motif (4 on the canvas).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_set("canvas", player:gauge("canvas") | 4)

    return effects
end
