-- Amplifier (Lv. 86): a Polyglot; only in Astral Fire or Umbral Ice.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local max = player.level >= 98 and 3 or 2
    if player:gauge("polyglot") < max then
        effects:gauge_add("polyglot", 1)
    end

    return effects
end
