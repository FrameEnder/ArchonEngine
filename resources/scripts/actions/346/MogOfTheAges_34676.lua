-- Mog Of The Ages (Lv. 30): 1,000 to the first enemy in a line, 70 % less to the rest; the Moogle Portrait spent.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 84, 1000 }, { 54, 800 }, { 1, 600 } }), 70)
    effects:gauge_set("creature", player:gauge("creature") & 47)

    return effects
end
