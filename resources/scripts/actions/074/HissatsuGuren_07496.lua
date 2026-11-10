-- Hissatsu Guren (Lv. 70): 400 to the enemies in a line; 25 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 400)
    effects:gauge_add("kenki", -25)

    return effects
end
