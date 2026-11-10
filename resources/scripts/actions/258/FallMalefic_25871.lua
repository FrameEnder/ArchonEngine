-- Fall Malefic (Lv. 82): 270.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, tierPotency(player, { { 94, 270 }, { 1, 250 } }))

    return effects
end
