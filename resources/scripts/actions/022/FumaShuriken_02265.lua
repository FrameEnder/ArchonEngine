-- Fuma Shuriken (Lv. 30): 500 (one mudra).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "fuma")

    return effects
end
