-- Hissatsu Gyoten (Lv. 54): 100 and a rush to the target; 10 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 100)
    effects:gauge_add("kenki", -10)

    return effects
end
