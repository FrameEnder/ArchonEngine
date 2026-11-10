-- Fuga (Lv. 26): 90 to the enemies in a cone; 5 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 90)
    samKenki(effects, player, 5)

    return effects
end
