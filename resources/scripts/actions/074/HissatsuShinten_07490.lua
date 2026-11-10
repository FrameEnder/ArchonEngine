-- Hissatsu Shinten (Lv. 52): 250; 25 Kenki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 250)
    effects:gauge_add("kenki", -25)

    return effects
end
