-- Contre Sixte (Lv. 56): 420 to the target and round it.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmMagic(effects, player, 420)

    return effects
end
