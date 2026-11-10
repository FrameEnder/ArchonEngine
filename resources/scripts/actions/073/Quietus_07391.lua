-- Quietus (Lv. 64): 240 to all enemies near; 50 Blood (none under Delirium before 96).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drkDamage(effects, player, 240)
    if player.level < 96 and player:get_effect(DRK_EFFECT_DELIRIUM) ~= nil then
        effects:consume_effect_stack(DRK_EFFECT_DELIRIUM)
    else
        effects:gauge_add("blood", -50)
    end
    drkWeaponskill(effects, player)

    return effects
end
