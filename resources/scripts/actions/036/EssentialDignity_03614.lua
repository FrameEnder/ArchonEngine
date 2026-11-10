-- Essential Dignity (Lv. 15): heals 400, up to 900 as the target's HP falls (here 400).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 400)

    return effects
end
