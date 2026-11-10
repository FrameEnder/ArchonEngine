-- Benefic I I (Lv. 26): heals 800.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 800)

    return effects
end
