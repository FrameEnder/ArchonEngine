-- Malefic I I (Lv. 54): 160.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, 160)

    return effects
end
