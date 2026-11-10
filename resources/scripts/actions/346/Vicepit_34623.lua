-- Vicepit (Lv. 70): 250 to all enemies near; a Rattling Coil from 82.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 250)
    if player.level >= 82 then
        effects:gauge_add("coils", 1)
    end

    return effects
end
