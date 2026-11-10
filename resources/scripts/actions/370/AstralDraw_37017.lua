-- Astral Draw (Lv. 30): The Balance, The Arrow and The Spire (Lord of Crowns from 70); 20 % of the MP back; Umbral Draw next.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:restore_mp(math.floor(player.max_mp / 5))
    effects:gauge_set("ast_draw", 1)
    effects:gauge_set("ast_cards", 1 | (3 << 4) | (6 << 8))
    if player.level >= 70 then
        effects:gauge_set("ast_arcana", 7)
    end

    return effects
end
