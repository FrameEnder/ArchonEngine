-- Fuko (Lv. 86): 100 to all enemies near; 10 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 100)
    samKenki(effects, player, 10)

    return effects
end
