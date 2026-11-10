-- Benefic (Lv. 2): heals 500.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 500)

    return effects
end
