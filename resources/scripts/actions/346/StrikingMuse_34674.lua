-- Striking Muse (Lv. 50): three stacks of Hammer Time, 30 s; the hammer off the canvas.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(PCT_EFFECT_HAMMER_TIME, 3, 30.0)
    effects:gauge_set("canvas", player:gauge("canvas") & 47)

    return effects
end
