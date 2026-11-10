-- Malefic I I I (Lv. 64): 190.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, 190)

    return effects
end
