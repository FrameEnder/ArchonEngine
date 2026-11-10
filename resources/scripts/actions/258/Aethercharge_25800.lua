-- Aethercharge (Lv. 6): the arcana: Ruby, from 15 Topaz too, from 22 all three.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local arcana = 1
    if player.level >= 22 then
        arcana = SMN_ARCANA_ALL
    elseif player.level >= 15 then
        arcana = 3
    end
    effects:gauge_set("arcana", arcana)

    return effects
end
