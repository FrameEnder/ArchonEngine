-- Hyoton (Lv. 45): 350 (two mudras ending in Jin).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "hyoton")

    return effects
end
