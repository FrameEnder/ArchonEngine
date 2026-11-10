-- Umbral Draw (Lv. 30): The Spear, The Bole and The Ewer (Lady of Crowns from 70); 20 % of the MP back; Astral Draw next.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:restore_mp(math.floor(player.max_mp / 5))
    effects:gauge_set("ast_draw", 0)
    effects:gauge_set("ast_cards", 4 | (2 << 4) | (5 << 8))
    if player.level >= 70 then
        effects:gauge_set("ast_arcana", 8)
    end

    return effects
end
