-- Hissatsu Kyuten (Lv. 62): 100 to all enemies near; 25 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 100)
    effects:gauge_add("kenki", -25)

    return effects
end
