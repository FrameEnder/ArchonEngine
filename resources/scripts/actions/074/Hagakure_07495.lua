-- Hagakure (Lv. 68): 10 Kenki for each Sen, the Sen gone.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local sen = player:gauge("sen")
    local count = (sen & 1) + ((sen >> 1) & 1) + ((sen >> 2) & 1)
    effects:gauge_add("kenki", 10 * count)
    effects:gauge_set("sen", 0)

    return effects
end
