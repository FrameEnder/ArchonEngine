-- Rabbit Medium (Lv. 30): a rabbit (a mudra repeated): nothing.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninCastNinjutsu(effects, player, "rabbit")

    return effects
end
