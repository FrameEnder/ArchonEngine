-- Retribution Of The Madeen (Lv. 96): 1,100 to the first enemy in a line, 70 % less to the rest; the Madeen Portrait spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, 1100, 70)
    effects:gauge_set("creature", player:gauge("creature") & 31)

    return effects
end
