-- Holy In White (Lv. 80): 570 to the target, 65 % less round it; a White Paint.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 570 }, { 84, 500 }, { 1, 460 } }), 65)
    effects:gauge_add("white_paint", -1)
    pctHue(effects, player, nil, nil)

    return effects
end
