-- Malefic I V (Lv. 72): 230.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, 230)

    return effects
end
