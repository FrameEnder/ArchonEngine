-- Raiton (Lv. 35): 740 (two mudras ending in Chi); a Raiju Ready stack (30 s) from 90.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "raiton")

    return effects
end
