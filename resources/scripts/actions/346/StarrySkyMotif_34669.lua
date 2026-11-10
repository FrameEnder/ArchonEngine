-- Starry Sky Motif (Lv. 70): paints the motif (32 on the canvas).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_set("canvas", player:gauge("canvas") | 32)

    return effects
end
