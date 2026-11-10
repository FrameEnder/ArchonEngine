-- Gyofu (Lv. 92): 240; 5 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 240)
    samKenki(effects, player, 5)

    return effects
end
